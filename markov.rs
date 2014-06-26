extern crate serialize;
extern crate time;

// use std::str;
// use std::rand;
use std::io::{File, BufferedReader};
use std::collections::HashMap;
use serialize::{json, Encodable, Decodable};

#[deriving(Decodable, Encodable, Show, Eq, PartialEq)]
pub struct MarkovModel {
    // nth order markov model has an n-piece 'state'
    order: uint,
    // a histogram of frequencies of sequences of length `order`
    frequencies: HashMap<String, uint>,
    // an optimization so we're not constantly re-counting
    total_occurences: uint
}

impl MarkovModel {
    pub fn new(order: uint) -> MarkovModel {
        MarkovModel {
            order: order,
            frequencies: HashMap::new(),
            total_occurences: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frequencies.is_empty()
    }
    
    // for appending to model files, training on a multi-file corpus
    pub fn load_or_create(filename: &str, order: uint) -> MarkovModel {
        if Path::new(filename).exists() {
            match MarkovModel::load(filename) {
                Ok(mm) => {
                    assert!(mm.order == order);
                    mm
                }
                Err(why) => {
                    // a little safety so i don't keep accidentally overwriting my book files
                    fail!("failed to load markov model from {}: {}", filename, why);
                }
            }
        } else {
            MarkovModel::new(order)
        }
    }

    pub fn load(filename: &str) -> Result<MarkovModel, json::DecoderError> {
        let mut f = match File::open(&Path::new(filename)) {
            Ok(f) => f,
            Err(why) => fail!("couldn't open file: {}", why)
        };
        let s = match f.read_to_str() {
            Ok(s) => s,
            Err(why) => fail!("couldn't read file: {}", why)
        };

        let json_object = json::from_str(s.as_slice());
        let mut decoder = json::Decoder::new(json_object.unwrap());

        // return Decodable::decode(&mut decoder).unwrap();
        let mm: MarkovModel = try!(Decodable::decode(&mut decoder));

        assert!(mm.total_occurences == mm.frequencies.values().fold(0, |a, &b| a + b));
        Ok(mm)
    }

    pub fn save(&self, filename: &str) {
        let mut f = match File::create(&Path::new(filename)) {
            Ok(f) => f,
            Err(why) => fail!("couldn't create file: {}", why)
        };

        // TODO: pretty json
        match f.write(json::Encoder::str_encode(self).as_bytes()) {
            Err(why) => fail!("couldn't write to file: {}", why),
            _ => ()
        };
    }

    fn inc_sequence_frequency(&mut self, key: &str) {
        self.frequencies.insert_or_update_with(
            key.to_string(),
            1,
            |_, v| {
                *v += 1;
            });
        self.total_occurences += 1;
    }

    fn set_frequency(&mut self, key: &str, freq: uint) {
        match self.frequencies.find(&key.to_string()) {
            Some(old_freq) =>  {
                self.total_occurences -= *old_freq;
            }
            None => {}
        };
        self.frequencies.insert(key.to_string(), freq);
        self.total_occurences += freq;
        assert!(self.total_occurences > 0);
    }

    pub fn train(&mut self, filename: &str) {
        let mut srcreader = match File::open(&Path::new(filename)) {
            Ok(srcfile) => { 
                BufferedReader::with_capacity(self.order * 500, srcfile) 
            }
            Err(_) => {
                println!("can't open source file {}", filename);
                return;
            }
        };

        let mut acc: String = "".to_string();
        loop {
            match srcreader.read_char() {
                Ok(c) => {
                    acc.push_char(c);
                    if acc.len() >= self.order {
                        self.inc_sequence_frequency(acc.as_slice());
                        acc.shift_char();
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    // choose a weighted random string out of the model, of length `self.order`
    pub fn generate_str<'a>(&'a self) -> &'a str {
        // ultra inefficient!
        // could just track this as we build the structure
        // let total_count: uint = self.frequencies.values().fold(0, |a, &b| a + b);

        if self.total_occurences == 0 {
            fail!("empty markov model! {}", self);
        }

        let n: uint = std::rand::random::<uint>() % self.total_occurences;
        let mut low: uint = 0;
        let mut high: uint = 0;

        for (k,v) in self.frequencies.iter() {
            high += *v;
            if low <= n && n < high {
                return k.as_slice();
            } 
            low = high;
        }

        fail!("somehow failed to generate an initial string");
    }

    pub fn generate_next_char(&self, prior: &str) -> char {
        // println!("generating for prior '{}'", prior);
        if prior.char_len() == 0 {
            // println!("empty prior, going with zero memory...");
            // reasonable way to get the first char of a string?
            match self.generate_str().chars().next() {
                Some(c) => c,
                None => fail!("couldn't generate a char somehow!")
            }
        } else {
            let sub = self.submodel(prior.slice_chars(1, prior.char_len()));
            if sub.is_empty() {
                // println!("empty submode for {}, cheaping out with shorter key '{}'", 
                //          prior, 
                //          prior.slice(1, prior.len()));

                // it's probably too late, as this state is an attractor.
                // try to salvage anyways.  recurse with less context.
                return self.generate_next_char(prior.slice_chars(1, prior.char_len()));
            }

            // println!("got submodel: {}", sub);
            match sub.generate_str().chars().last() {
                Some(c) => c,
                None => fail!("couldn't generate char from submodel!")
            }
        }
    }

    // TODO: ideally we privatize the generation fns above and have one that returns an iterator

    /** Produce a subset of the model for values with the given prefix.
     */
    fn submodel(&self, prefix: &str) -> MarkovModel {
        // let t_before = time::precise_time_s();
        // seems like we frequently produce empty submodels...
        let mut mm = MarkovModel::new(self.order);
        
        for (k, v) in self.frequencies.iter() {
            let kslice = k.as_slice();
            if kslice.starts_with(prefix) {
                mm.set_frequency(kslice, *v);
            }
        }
        // seems like it takes about 1 second to build a submodel for a 2.5 million entry model
        // maybe i can switch to a TreeMap and use lower_bound() to speed this up
        // println!("submodel for '{}' of size {} (full {}) built in {} secs.", 
        //          prefix, 
        //          mm.frequencies.len(), 
        //          self.frequencies.len(), 
        //          time::precise_time_s() - t_before);
        return mm;
    }

}

fn train_to_file(order: uint, dbfilename: &str, srcfilename: &str) {
    let mut mm = MarkovModel::load_or_create(dbfilename, order);
    mm.train(srcfilename);
    mm.save(dbfilename);
}

fn generate_from_db(dbfilename: &str) {
    print!("loading db...");
    let t_before = time::precise_time_s();
    std::io::stdio::flush();
    let mm = match MarkovModel::load(dbfilename) {
        Ok(mm) => mm,
        Err(why) => fail!("couldn't decode MarkovModel: {}", why)
    };
    println!("loaded in {} secs.", time::precise_time_s() - t_before);

    // produce an infinite stream of results
    let mut prior: String = mm.generate_str().to_string();
    print!("{}", prior);
    loop {
        let c = mm.generate_next_char(prior.as_slice());
        // drop head of prior, append c
        prior.shift_char();
        prior.push_char(c);
        print!("{}", c);
        std::io::stdio::flush();
    }
}

fn main() {
    let args = std::os::args();

    // todo: better option parsing
    // unfortunately rust getopt seems pretty weak
    assert!(args.len() > 2);
    match args.get(1).as_slice() {
        // markov train order dbfile sourcefile
        "train" => {
            assert!(args.len() == 5);
            let order = match from_str(args.get(2).as_slice()) {
                Some(n) => n,
                None => fail!("order must be an integer. you gave '{}'", args.get(2))
            };
            train_to_file(order, 
                          args.get(3).as_slice(), 
                          args.get(4).as_slice());
        }
        // markov generate dbfile
        "generate" => {
            generate_from_db(args.get(2).as_slice());
        }
        cmd => {
            println!("don't know how to handle command {}", cmd);
        }
    }
}
