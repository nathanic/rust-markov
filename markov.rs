extern crate serialize;
use serialize::{json, Encodable, Decodable};
use std::io::File;

use std::str;
use std::rand;
use std::io::{File, BufferedReader};
use std::collections::HashMap;

type WordCounts = HashMap<String, uint>;

#[deriving(Decodable, Encodable, Show, Eq, PartialEq)]
pub struct MarkovModel {
    order: uint,
    frequencies: HashMap<String, uint>,
}

impl MarkovModel {
    pub fn new(order: uint) -> MarkovModel {
        MarkovModel {
            order: order,
            frequencies: HashMap::new()
        }
    }

    pub fn load_or_create(filename: &str, order: uint) -> MarkovModel {
        if Path::new(filename).exists() {
            let mm = MarkovModel::load(filename);
            assert!(mm.order == order);
            mm
        } else {
            MarkovModel::new(order)
        }
    }

    pub fn load(filename: &str) -> MarkovModel {
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

        return Decodable::decode(&mut decoder).unwrap();
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

    // can i attach this to my wordcounts type as a member?
    pub fn inc_word_count(&mut self, key: &str) {
        self.frequencies.insert_or_update_with(
            key.to_string(),
            1,
            |_, v| {
                *v += 1;
            });
        // self.total_occurences += 1;
    }

    pub fn set_frequency(&mut self, key: &str, freq: uint) {
        let mut old_freq = self.frequencies.find_or_insert(key.to_string(), freq);
        *old_freq = freq;
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

        loop {
            match srcreader.fill_buf() {
                Ok(buf) => {
                    if buf.len() >= self.order {
                        // flawed, probably loses the ends of every buffer
                        // also totally fails to deal with utf-8
                        let slice = buf.slice(0, self.order);
                        match str::from_utf8(slice) {
                            Some(s) => self.inc_word_count(s),
                            None => { println!("WARNING: got some broken utf-8: {}", slice); }
                        };
                    }
                }
                Err(_) => {
                    break;
                }
            }
            srcreader.consume(1);
        }
    }

    pub fn generate_str<'a>(&'a self) -> &'a str {
        // ultra inefficient!
        // could just track this as we build the structure
        let total_count: uint = self.frequencies.values().fold(0, |a, &b| a + b);
        if total_count == 0 {
            fail!("empty markov model! {}", self);
        }
        let n: uint = std::rand::random::<uint>() % total_count;

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
        if prior.char_len() == 0 {
            // reasonable way to get the first char of a string?
            match self.generate_str().chars().next() {
                Some(c) => c,
                None => fail!("couldn't generate a char somehow!")
            }
        } else {
            let sub = self.submodel(prior.slice(1, prior.len()));
            println!("got submodel: {}", sub);
            match sub.generate_str().chars().next() {
                Some(c) => c,
                None => fail!("couldn't generate char from submodel!")
            }
        }
    }

    // TODO: ideally we privatize the generation fns above and have one that returns an iterator

    /** Produce a subset of the model for values with the given prefix.
     */
    fn submodel(&self, prefix: &str) -> MarkovModel {
        // seems like we frequently produce empty submodels...
        let mut mm = MarkovModel::new(self.order);
        
        for (k, v) in self.frequencies.iter() {
            let kslice = k.as_slice();
            if kslice.starts_with(prefix) {
                mm.set_frequency(kslice, *v);
            }
        }
        return mm;
    }

}

fn train_to_new_file(order: uint, dbfilename: &str, srcfilename: &str) {
    let mut mm = MarkovModel::new(order);
    mm.train(srcfilename);
    mm.save(dbfilename);
}

fn generate_from_db(dbfilename: &str) {
    println!("pretend i'm generating from file {}", dbfilename);
    let mm = MarkovModel::load(dbfilename);

    // produce an infinite stream of results
    let mut prior: String = mm.generate_str().to_string();
    print!("{}", prior);
    loop {
        let c = mm.generate_next_char(prior.as_slice());
        // drop head of prior, append c
        prior.shift_char();
        prior.push_char(c);
        print!("{}", c);
    }
}

fn main() {
    let args = std::os::args();

    assert!(args.len() > 2);
    // todo: better option parsing
    match args.get(1).as_slice() {
        // markov train order dbfile sourcefile
        "train" => {
            assert!(args.len() == 5);
            let order = match from_str(args.get(2).as_slice()) {
                Some(n) => n,
                None => fail!("order must be an integer. you gave '{}'", args.get(2))
            };
            train_to_new_file(order, args.get(3).as_slice(), args.get(4).as_slice());
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
