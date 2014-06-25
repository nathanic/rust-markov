// figuring out triemap
// 2014-06-24 

use std::collections::HashMap;

#[deriving(Show,Eq,PartialEq)]
struct Trie<T> {
    value: Option<T>,
    children: HashMap<char, Box<Trie<T>>>
}

impl<T: Clone> Trie<T> {
    pub fn new() -> Trie<T> {
        Trie { 
            value: None,
            // is HashMap the best option for indexing by single chars?
            // good enough to start with anyway...
            children: HashMap::new()
        }
    }

    pub fn find(&self, key: &str) -> Option<T> {
        let first = match key.chars().next() {
            Some(c) => c,
            None => fail!("find on zero-length key!")
        };

        match self.children.find(&first) {
            Some(subtrie) => {
                if key.len() == 0 {
                    subtrie.value.clone()
                    // or (*subtrie).value, since it's a hashmap of boxes of tries?
                } else {
                    subtrie.find(key.slice_chars(1, key.char_len()))
                }
            }
            None => None
        }
    }

    pub fn insert(&mut self, key: &str, val: T) {
        let mut chars = key.chars();
        let first = match chars.next() {
            Some(c) => c,
            None => fail!("insert on zero-length key!")
        };

        let mut cur = self;
        for c in chars {
            println!("got a char {}", c);
            match cur.children.find(&c) {
                Some(child) => {
                    cur = child;
                }
                None => {
                    let newborn = box Trie::new();
                    cur.children.insert(c, newborn);
                    cur = newborn;
                }
            }
        }

        // really need to do a pointer compare, not structural...
        if cur != self {
            cur.value = Some(val);
        }

        // match self.children.find(first) {
        //     Some(subtrie) => {
        //         // this is probably unicode-unsafe!
        //         subtrie.insert(key.slice(1, key.len() -1), val);
        //     }
        //     None => {
        //         let mut subtrie = box Trie::new();
        //         self.children.insert(first, subtrie);
        //     }
        // }
    }
}


fn main() {
    let mut trie: Trie<uint> = Trie::new();
    trie.insert("hello", 2);

}
