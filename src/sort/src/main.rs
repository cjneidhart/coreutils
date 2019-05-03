use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let mut args = env::args();
    let filename = args.nth(1).unwrap_or("-".to_string());
    if filename == "-" {
        process_file(BufReader::new(io::stdin()))
    } else {
        process_file(BufReader::new(File::open(filename)?))
    }
}

fn process_file<R: BufRead>(file: R) -> io::Result<()> {
    // let mut lines: Vec<_> = file.lines().map(|x| x?).collect();
    let mut lines = BTreeMap::new();
    for line_result in file.lines() {
        let line = line_result?;
        let new_count = match lines.get(&line) {
            Some(n) => n + 1,
            None => 1,
        };
        lines.insert(line, new_count);
    }

    for (line, count) in lines {
        for _ in 0..count {
            println!("{}", line);
        }
    }

    Ok(())
}
