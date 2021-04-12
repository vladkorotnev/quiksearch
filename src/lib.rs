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
pub type FuzzyDict<T> = WordListNode<T>;
pub type WordDict = FuzzyDict<String>;

/// Priority for fuzzy algorithm
pub enum FuzzPriority {
    /// When an unexpected character is found, prefer to treat it as a word boundary
    WordBoundary,
    /// When an unexpected character is found, prefer to treat it as a typo. Falls back to WordBoundary if nothing is found, thus may be slower, but more precise.
    TypoCorrection
}

/// Search algorithm choice
pub enum SearchKind {
    /// Search for an exact match
    Strict,
    /// Search for a prefix match with specified depth
    Prefix(usize),
    /// Search for a fuzzy prefix match with specified fuzz
    Fuzzy(usize, FuzzPriority)
}

pub struct WordListNode<Term> where Term: std::cmp::Eq + std::hash::Hash {
    // Contains pointers to terms
    terms: HashSet<Rc<Term>>,
    children: HashMap<Letter, Self>
}

impl<T: std::cmp::Eq + std::hash::Hash + std::fmt::Debug> WordListNode<T> {
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
    pub fn learn_term(&mut self, term_repr: Rc<String>, term: Rc<T>) {
        for word in term_repr.split(|chr: char| !chr.is_alphanumeric()) {
            // Create a branch for the current word
            self.learn_word(word)
            // And add the term to it's end node
                .terms.insert(term.clone());
        }

        let no_spaces = term_repr.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
        self.learn_word(&no_spaces).terms.insert(term);
    }

    /// Collect all the terms from this node and down to the specified node depth (Recursive)
    fn collect_terms(&self, depth: Option<usize>) -> Vec<Rc<T>> {
        let mut terms: Vec<Rc<T>> = self.terms.iter().map(|r| r.clone()).collect();
        
        // If depth is provided, only go as far as that depth
        if let Some(depth) = depth {
            if depth > 0 {
                for (_, child) in self.children.iter() {
                    terms.append(&mut child.collect_terms(Some(depth - 1)));
                }
            }
        }
        // Otherwise go as deep as first matches only
        else {
            // try to find from the nodes below
            for (_, child) in self.children.iter() {
                terms.append(&mut child.collect_terms(None));
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
    pub fn find_terms(&self, query: &str, kind: SearchKind) -> Vec<Rc<T>> {
        use std::iter::FromIterator;

        let mut now_node = self;
        let max_i = query.len() - 1;
        let lower_query: String = query.to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
        let mut restrict_to: HashSet<Rc<T>> = HashSet::new();

        for (i, c) in  lower_query.chars().enumerate() {
            match now_node.children.get(&c) {
                None => {
                    if i != max_i {
                        // If we ended up here, then it's a mismatched letter mid-word.
                        match kind {
                            SearchKind::Fuzzy(fuzz, ref pri) => {
                                // try to assume we reached a word boundary

                                // to avoid false positives, restrict to those which would have appeared if we continued to match, but only once
                                if restrict_to.len() == 0 {
                                    let could_have_been = now_node.collect_terms(None);
                                    restrict_to.extend(could_have_been.into_iter());
                                }

                                match pri {
                                    FuzzPriority::WordBoundary => {
                                        // Try to find new word beginning with current char
                                        match self.hope_for_success(&c, 1) {
                                            Some(alt_word) => {
                                                let new_candidates = HashSet::from_iter(alt_word.collect_terms(None).into_iter());
                                                if new_candidates.intersection(&restrict_to).count() > 0 {
                                                    now_node = alt_word; // found alternate node in another word among current results, continue from there
                                                    continue;
                                                }
                                                // else fall through to typo correction
                                            }, 
                                            None => () // fall through to typo correction
                                        }
                                    }

                                    FuzzPriority::TypoCorrection => {
                                        match now_node.hope_for_success(&c, fuzz) {
                                            Some(alt_node) => {
                                                // found an alternate node matching current char, continue from there
                                                now_node = alt_node;
                                            }, 
                                            None => {
                                                // found no alternate node further, last hope is try from the root
                                                match self.hope_for_success(&c, 1) {
                                                    Some(alt_word) => {
                                                        now_node = alt_word;
                                                    }, // found alternate node in another word, continue from there
                                                    None => ()
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            _ => return vec![] // not fuzzy search, give up
                        }
                    }
                },
                Some(x) => now_node = x
            }
        }
        
        let depth_limit = match kind {
            SearchKind::Fuzzy(_, _) => None,
            SearchKind::Prefix(depth) => Some(depth),
            SearchKind::Strict => Some(0)
        };
        let res = now_node.collect_terms(depth_limit);

        let rslt: Vec<Rc<T>> = res.into_iter()
                        .unique()
                        .filter(|x| if restrict_to.len() > 0 { restrict_to.contains(x) } else { true } )
                        .collect();
        if rslt.len() == 0 {
            match kind {
                SearchKind::Fuzzy(fuzz, pri) => match pri {
                    FuzzPriority::TypoCorrection =>  return self.find_terms(query, SearchKind::Fuzzy(fuzz, FuzzPriority::WordBoundary)),
                    _ => ()
                }
                _ => ()
            }
        }
        rslt
    }
}

impl WordDict {
    pub fn learn(&mut self, term: String) {
        let rc = Rc::new(term);
        self.learn_term(rc.clone(), rc);
    }
}


#[cfg(test)]
mod tests {
    use super::{WordDict, SearchKind, FuzzPriority};

    #[test]
    fn it_saves_strings_strict() {
        let mut dict = WordDict::new();

        dict.learn(String::from("hello"));
        
        // Expect to find "hello"
        assert!( dict.find_terms("hello", SearchKind::Strict).len() > 0 );
        // Expect to find only "hello"
        assert!( dict.find_terms("hello", SearchKind::Strict).len() == 1 );
        // Expect to find "hello" by prefix
        assert!( dict.find_terms("hell", SearchKind::Prefix(10)).len() > 0 );
        // Expect to not find "hell" as a word
        assert!( dict.find_terms("hell", SearchKind::Strict).len() == 0 );

        dict.learn(String::from("hell"));
        // Expect to find "hell" as a word and not hello
        assert!( dict.find_terms("hell", SearchKind::Strict).len() > 0 );
        assert!( dict.find_terms("hell", SearchKind::Strict).len() == 1 );
        // Expect to find "hell" and "hello" by prefix
        assert!( dict.find_terms("hell", SearchKind::Prefix(10)).len() == 2 );
        assert!( dict.find_terms("he", SearchKind::Prefix(10)).len() == 2 );

        // But if depth is too shallow, find nothing
        assert!( dict.find_terms("he", SearchKind::Prefix(1)).len() == 0 );

        // Expect to not find what we didn't save
        assert!( dict.find_terms("hejkjk", SearchKind::Prefix(10)).len() == 0 );
        assert!( dict.find_terms("obama", SearchKind::Prefix(10)).len() == 0 );
        assert!( dict.find_terms("ajdklajhf", SearchKind::Prefix(10)).len() == 0 );
    }

    #[test]
    fn it_searches_by_words() {
        let mut dict = WordDict::new();

        dict.learn(String::from("Hello World"));
        dict.learn(String::from("World Is Mine"));
        dict.learn(String::from("miku miku ni shite ageru"));

        fn check_finds(dict: &WordDict, query: &str, kind: SearchKind) {
            let rslt = dict.find_terms(query, kind);
            println!("{}: {:#?}", query, rslt);
            assert!( rslt.len() > 0 );
        }

        check_finds(&dict, "world", SearchKind::Strict);
        check_finds(&dict, "mi", SearchKind::Prefix(10));
        check_finds(&dict, "hello", SearchKind::Strict);
    }

    #[test]
    fn it_searches_fuzzy() {
        const FUZZ: usize = 5;

        let mut dict = WordDict::new();

        dict.learn(String::from("Hello World"));
        dict.learn(String::from("World Is Mine"));
        dict.learn(String::from("miku miku ni shite ageru"));

        fn check_finds(dict: &WordDict, query: &str, kind: SearchKind, expect: &str) {
            use std::borrow::Borrow;

            let rslt = dict.find_terms(query, kind);
            println!("{}: {:#?}", query, rslt);
            assert!( rslt.len() > 0 );
            assert!( rslt.iter().map(|r| r.borrow() ).any(|v: &String| v.eq(expect)) );
        }

        check_finds(&dict, "helwor", SearchKind::Fuzzy(FUZZ, FuzzPriority::TypoCorrection), "Hello World");
        check_finds(&dict, "miminishiage", SearchKind::Fuzzy(FUZZ, FuzzPriority::TypoCorrection), "miku miku ni shite ageru");
        check_finds(&dict, "woismi", SearchKind::Fuzzy(FUZZ, FuzzPriority::TypoCorrection), "World Is Mine");
    }
}
