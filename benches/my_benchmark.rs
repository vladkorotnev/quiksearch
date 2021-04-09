use criterion::{criterion_group, criterion_main, Criterion, black_box};
use quiksearch::{WordDict, SearchKind};
use std::rc::Rc;

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("benches");
    
    g.sample_size(10);
    g.warm_up_time(std::time::Duration::from_secs(11));

    g.bench_function("learn all words", |b| {
        use std::fs::File;
        use std::io::{self, BufRead};

        b.iter(|| {
            let file = File::open("assets/test_list.txt").unwrap();
            let words_list = io::BufReader::new(file).lines();
            let mut dict = WordDict::new();

            for line in words_list {
                if let Ok(line) = line {
                    dict.learn_term(Rc::new(line));
                }
            }
        });
    });

    g.warm_up_time(std::time::Duration::from_secs(3));
    g.sample_size(100);

    g.bench_function("exact matching", |b| {
        use std::fs::File;
        use std::io::{self, BufRead};
        use rand::seq::SliceRandom;

        let file = File::open("assets/test_list.txt").unwrap();
        let words_list = io::BufReader::new(file).lines();
        let mut dict = WordDict::new();
        let mut terms = vec![];

        for line in words_list {
            if let Ok(line) = line {
                let term = Rc::new(line);
                dict.learn_term(term.clone());
                terms.push(term);
            }
        }
        
        b.iter(|| {
            let term = terms.choose(&mut rand::thread_rng()).unwrap();
            let _ = black_box(dict.find_terms(term, SearchKind::Strict));
        });
    });

    g.bench_function("prefix matching", |b| {
        use std::fs::File;
        use std::io::{self, BufRead};
        use rand::{distributions::Alphanumeric, Rng};

        let file = File::open("assets/test_list.txt").unwrap();
        let words_list = io::BufReader::new(file).lines();
        let mut dict = WordDict::new();
        for line in words_list {
            if let Ok(line) = line {
                let term = Rc::new(line);
                dict.learn_term(term.clone());
            }
        }
        
        b.iter(|| {
            let term: String = rand::thread_rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(3)
                                    .map(char::from)
                                    .collect();
            let _ = black_box(dict.find_terms(&term, SearchKind::Prefix(3)));
        });
    });

    g.bench_function("fuzzy matching", |b| {
        use std::fs::File;
        use std::io::{self, BufRead};
        use rand::{distributions::Alphanumeric, Rng};

        let file = File::open("assets/test_list.txt").unwrap();
        let words_list = io::BufReader::new(file).lines();
        let mut dict = WordDict::new();
        for line in words_list {
            if let Ok(line) = line {
                let term = Rc::new(line);
                dict.learn_term(term.clone());
            }
        }
        
        b.iter(|| {
            let term: String = rand::thread_rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(3)
                                    .map(char::from)
                                    .collect();
            let _ = black_box(dict.find_terms(&term, SearchKind::Fuzzy(3, 5)));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);