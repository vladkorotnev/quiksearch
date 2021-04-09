use quiksearch::{WordDict, SearchKind};
use std::fs::File;
use std::io::{self, BufRead, stdout, Write};
use std::time::Instant;

fn main() {
    println!("Predictive Search");


    let start = Instant::now();

    let file = File::open("assets/test_list.txt").unwrap();
    let words_list = io::BufReader::new(file).lines();
    let mut dict = WordDict::new();
    let mut counter = 0;

    for line in words_list {
        if let Ok(line) = line {
            dict.learn(line);
            counter += 1;
        }
    }

    println!("Loading took {:?}, loaded {} terms", start.elapsed(), counter);

    let mut input;
    println!("Enter query without whitespace, Ctl-C to exit.");

    const FUZZ: usize = 7;

    loop {
        input = String::from("");

        print!("> ");
        stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("error: unable to read user input");
       
        // filter input
        input = input.chars().filter(|c| c.is_alphanumeric()).collect();

        let start = Instant::now();
        let rslt = dict.find_terms(&input.trim(), SearchKind::Fuzzy(FUZZ));
        println!("Search took {:?}", start.elapsed());

        if rslt.len() > 0  {
            println!("You might have meant: {:#?}", rslt);
        } else {
            println!("No result for {}", input);
        }
    }
}
