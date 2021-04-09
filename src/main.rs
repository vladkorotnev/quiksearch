use quiksearch::{WordDict, SearchKind};
use std::fs::File;
use std::io::{self, BufRead, stdout, Write};
use std::rc::Rc;
use std::time::Instant;

fn main() {
    println!("Predictive Search");


    let start = Instant::now();

    let file = File::open("assets/test_list.txt").unwrap();
    let words_list = io::BufReader::new(file).lines();
    let mut dict = WordDict::new();

    for line in words_list {
        if let Ok(line) = line {
            let term = Rc::new(line);
            dict.learn_term(term.clone());
        }
    }

    println!("Loading took {:?}", start.elapsed());

    let mut input;
    println!("Enter query, Ctl-C to exit.");

    const FUZZ: usize = 5;
    const DEPTH: usize = 30;

    loop {
        input = String::from("");

        print!("> ");
        stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("error: unable to read user input");
       
        let start = Instant::now();
        if let Some(rslt) = dict.find_terms(&input.trim(), SearchKind::Fuzzy(DEPTH, FUZZ)) {
            println!("You might have meant: {:#?}", rslt);
        } else {
            println!("No result for {}", input);
        }
        println!("Search took {:?}", start.elapsed());
    }
}
