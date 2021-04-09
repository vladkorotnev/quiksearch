use criterion::{criterion_group, criterion_main, Criterion, black_box};
use quiksearch::{WordDict, SearchKind};

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
                    dict.learn(line);
                }
            }
        });
    });

    g.warm_up_time(std::time::Duration::from_secs(3));
    g.sample_size(1000);

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
                dict.learn(line.clone());
                terms.push(line);
            }
        }
        
        b.iter(|| {
            let term: String = terms.choose(&mut rand::thread_rng()).unwrap().chars().filter(|x| x.is_alphanumeric()).collect();
            let _ = black_box(dict.find_terms(&term, SearchKind::Strict));
        });
    });

    g.bench_function("prefix matching gibberish", |b| {
        use std::fs::File;
        use std::io::{self, BufRead};
        use rand::{distributions::Alphanumeric, Rng};

        let file = File::open("assets/test_list.txt").unwrap();
        let words_list = io::BufReader::new(file).lines();
        let mut dict = WordDict::new();
        for line in words_list {
            if let Ok(line) = line {
                dict.learn(line);
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

    g.bench_function("fuzzy matching gibberish", |b| {
        use std::fs::File;
        use std::io::{self, BufRead};
        use rand::{distributions::Alphanumeric, Rng};

        let file = File::open("assets/test_list.txt").unwrap();
        let words_list = io::BufReader::new(file).lines();
        let mut dict = WordDict::new();
        for line in words_list {
            if let Ok(line) = line {
                dict.learn(line);
            }
        }
        
        b.iter(|| {
            let term: String = rand::thread_rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(3)
                                    .map(char::from)
                                    .collect();
            let _ = black_box(dict.find_terms(&term, SearchKind::Fuzzy(5)));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);