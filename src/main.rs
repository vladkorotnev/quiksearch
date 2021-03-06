use quiksearch::{FuzzPriority, SearchKind, WordDict};
use std::fs::File;
use std::io::{self, stdout, BufRead, Write};
use std::time::Instant;

fn main() {
    println!("Predictive Search");

    let start = Instant::now();

    let file = File::open("assets/test_list.txt").unwrap();
    let words_list = io::BufReader::new(file).lines();
    let mut dict = WordDict::new();
    let mut counter: u32 = 0;

    for line in words_list {
        if let Ok(line) = line {
            dict.learn(line);
            counter += 1;
        }
    }

    let load_time = start.elapsed();
    println!(
        "Loading took {:?}, loaded {} terms: about {} terms per second",
        load_time,
        counter,
        (counter as f64 / load_time.as_secs_f64()).floor()
    );

    let mut input;
    println!("Enter query without whitespace, Ctl-C to exit.");

    const FUZZ: usize = 3;

    loop {
        input = String::from("");

        print!("> ");
        stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");

        let start = Instant::now();
        let rslt = dict.find_terms(
            &input.trim(),
            SearchKind::Fuzzy(FUZZ, FuzzPriority::TypoCorrection),
        );
        println!("Search took {:?}", start.elapsed());

        if rslt.len() > 0 {
            println!("You might have meant: {:#?}", rslt);
        } else {
            println!("No result for {}", input);
        }
    }
}
