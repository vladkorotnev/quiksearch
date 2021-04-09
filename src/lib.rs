extern crate itertools;
use itertools::Itertools;


/// QuickSilver-esque word matching algorithm
/// 
/// Abstract:
/// Remember the famous Mac application "QuickSilver"? One of the massive advantages of it was "abbreviations",
/// which allowed you to search for an app or file by typing a mnemonic for it. For example, if I had the following apps:
/// * Photos
/// * Photo Booth
/// * Adobe Photoshop
/// * Photo Magic
/// I could launch Photoshop by typing "phosh" (because PHOtoSHop), Photo Booth by typing "phobo" (because PHOto BOoth), and so forth.
/// 
/// This is an attempt to create an algorithm which would be able to index a list of terms and provide a similar search ability with decent performance. 
/// 
/// As a "term" we shall use an arbitrary string consisting of "words" separated by whitespace. 
/// As a "query" we shall use an arbitrary string, case-insensitive, without any whitespace at all (because in QuickSilver spacebar was used to show more results).
/// The goal would be for the algorithm to return a list of the terms the user possibly wanted, ranked by similarity, keeping the original case and spelling exactly as when the term was learned by the database.
/// 

use std::rc::Rc;
use std::collections::{HashMap, HashSet};

pub type Letter = char;
pub type WordDict = WordListNode;

/// Search algorithm choice
pub enum SearchKind {
    /// Search for an exact match
    Strict,
    /// Search for a prefix match with specified depth
    Prefix(usize),
    /// Search for a fuzzy prefix match with specified depth and fuzz
    Fuzzy(usize, usize)
}

pub struct WordListNode {
    // Contains pointers to terms
    terms: HashSet<Rc<String>>,
    children: HashMap<Letter, Self>
}

impl WordListNode {
    /// Creates an empty wordlist node
    pub fn new() -> Self {
        Self {
            terms: HashSet::new(),
            children: HashMap::new()
        }
    }

    /// Learns a single word. Returns the final node of the word in the word list.
    fn learn_word(&mut self, term: &str) -> &mut Self {
        // Get a reference of the current letter node
        let mut modify_node = self;

        for c in term.to_lowercase().chars() {
            if c.is_alphanumeric() {
                 // Add a child node and replace reference to current letter node
                modify_node = modify_node.children.entry(c).or_insert(Self::new());
            }
        }

        // Return reference to last node of word
        modify_node
    }

    /// Learns a single term, which may consist of multiple words, separated by whitespace
    pub fn learn_term(&mut self, term: Rc<String>) {
        for word in term.split_whitespace() {
            // Create a branch for the current word
            self.learn_word(word)
            // And add the term to it's end node
                .terms.insert(term.clone());
        }

        let no_spaces = term.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        self.learn_word(&no_spaces).terms.insert(term);
    }

    /// Collect all the terms from this node and down to the specified node depth (Recursive)
    fn collect_terms(&self, depth: usize) -> Vec<Rc<String>> {
        let mut terms: Vec<Rc<String>> = self.terms.iter().map(|r| r.clone()).collect();
        if depth > 0 {
            for (_, child) in self.children.iter() {
                terms.append(&mut child.collect_terms(depth - 1));
            }
        }
        terms
    }

    /// Try to greedy find the next node that could match a character
    fn hope_for_success(&self, chara: &char, fuzz: usize) -> Option<&Self> {
        if fuzz > 0 {
            for (child_char, child) in self.children.iter() {
                if child_char == chara {
                    return Some(child)
                }

                match child.hope_for_success(chara, fuzz - 1) {
                    Some(rslt) => return Some(rslt),
                    _ => ()
                }
            }
        }
        None
    }

    /// Perform a strict query prefix search with specified depth fuzz
    pub fn find_terms(&self, query: &str, kind: SearchKind) -> Option<Vec<Rc<String>>> {
        let mut now_node = self;
        let max_i = query.len() - 1;
        let lower_query = query.to_lowercase();

        for (i, c) in  lower_query.chars().enumerate() {
            assert!(!c.is_whitespace());
            match now_node.children.get(&c) {
                None => {
                    if i != max_i {
                        // If we ended up here, then it's a mismatched letter mid-word.
                        match kind {
                            SearchKind::Fuzzy(depth, fuzz) => {
                                match now_node.hope_for_success(&c, fuzz) {
                                    Some(alt_node) => {
                                        // found an alternate node matching current char, continue from there
                                        now_node = alt_node;
                                    }, 
                                    None => {
                                        // found no alternate node further, last hope is try from the root

                                        // TODO: SCRAP THE FOLLOWING? Seems to improve false positives
                                        // match self.hope_for_success(&c, 1) {
                                        //     Some(alt_word) => {
                                        //         now_node = alt_word;
                                        //     }, // found alternate node in another word, continue from there
                                        //     None => return None // found no alternative, give up
                                        // }
                                    }
                                }
                            },
                            _ => return None // not fuzzy search, give up
                        }
                    }
                },
                Some(x) => now_node = x
            }
        }
        
        let depth_limit = match kind {
            SearchKind::Fuzzy(depth, _) => depth,
            SearchKind::Prefix(depth) => depth,
            SearchKind::Strict => 1
        };

        let rslt: Vec<Rc<String>> = now_node.collect_terms(depth_limit).into_iter().unique().collect();
        if rslt.len() == 0 {
            None
        } else {
            Some(rslt)
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::{WordDict, SearchKind};
    use std::rc::Rc;

    #[test]
    fn it_saves_strings_strict() {
        let mut dict = WordDict::new();

        dict.learn_term(Rc::new(String::from("hello")));
        
        // Expect to find "hello"
        assert!( dict.find_terms("hello", SearchKind::Strict).is_some() );
        // Expect to find only "hello"
        assert!( dict.find_terms("hello", SearchKind::Strict).unwrap().len() == 1 );
        // Expect to find "hello" by prefix
        assert!( dict.find_terms("hell", SearchKind::Prefix(10)).is_some() );
        // Expect to not find "hell" as a word
        assert!( dict.find_terms("hell", SearchKind::Strict).is_none() );

        dict.learn_term(Rc::new(String::from("hell")));
        // Expect to find "hell" as a word and not hello
        assert!( dict.find_terms("hell", SearchKind::Strict).is_some() );
        assert!( dict.find_terms("hell", SearchKind::Strict).unwrap().len() == 1 );
        // Expect to find "hell" and "hello" by prefix
        assert!( dict.find_terms("hell", SearchKind::Prefix(10)).unwrap().len() == 2 );
        assert!( dict.find_terms("he", SearchKind::Prefix(10)).unwrap().len() == 2 );

        // But if depth is too shallow, find nothing
        assert!( dict.find_terms("he", SearchKind::Prefix(1)).is_none() );

        // Expect to not find what we didn't save
        assert!( dict.find_terms("hejkjk", SearchKind::Prefix(10)).is_none() );
        assert!( dict.find_terms("obama", SearchKind::Prefix(10)).is_none() );
        assert!( dict.find_terms("ajdklajhf", SearchKind::Prefix(10)).is_none() );
    }

    #[test]
    fn it_searches_by_words() {
        let mut dict = WordDict::new();

        dict.learn_term(Rc::new(String::from("Hello World")));
        dict.learn_term(Rc::new(String::from("World Is Mine")));
        dict.learn_term(Rc::new(String::from("miku miku ni shite ageru")));

        fn check_finds(dict: &WordDict, query: &str, kind: SearchKind) {
            let rslt = dict.find_terms(query, kind);
            println!("{}: {:#?}", query, rslt);
            assert!( rslt.is_some() );
        }

        check_finds(&dict, "world", SearchKind::Strict);
        check_finds(&dict, "mi", SearchKind::Prefix(10));
        check_finds(&dict, "hello", SearchKind::Strict);
    }

    #[test]
    fn it_searches_fuzzy() {
        let mut dict = WordDict::new();

        dict.learn_term(Rc::new(String::from("Hello World")));
        dict.learn_term(Rc::new(String::from("World Is Mine")));
        dict.learn_term(Rc::new(String::from("miku miku ni shite ageru")));

        fn check_finds(dict: &WordDict, query: &str, kind: SearchKind, expect: &str) {
            use std::borrow::Borrow;

            let rslt = dict.find_terms(query, kind);
            println!("{}: {:#?}", query, rslt);
            assert!( rslt.is_some() );
            assert!( rslt.unwrap().iter().map(|r| r.borrow() ).any(|v: &String| v.eq(expect)) );
        }

        check_finds(&dict, "helwor", SearchKind::Fuzzy(10, 10), "Hello World");
        check_finds(&dict, "miminishiage", SearchKind::Fuzzy(10, 10), "miku miku ni shite ageru");
        check_finds(&dict, "woismi", SearchKind::Fuzzy(10, 10), "World Is Mine");
    }
}
