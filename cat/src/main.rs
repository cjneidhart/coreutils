extern crate clap;

use std::process;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};

use clap::{Arg, App};

#[derive(Default)]
struct Args {
    number_nonblank: bool,
    number: bool,
    squeeze_blank: bool,
    show_ends: bool,
    show_tabs: bool,
    show_nonprinting: bool,
}

impl Args {
    fn simple(&self) -> bool {
        !(self.number_nonblank || self.number || self.squeeze_blank
          || self.show_ends || self.show_tabs || self.show_nonprinting)
    }
}

fn main() {
    let (args, filenames) = parse_args();
    let mut success = true;

    if args.simple() {
        for filename in filenames {
            if filename == "-" {
                let mut stdin = io::stdin();
                run_file_simple(&mut stdin);
            } else {
                let mut file = match File::open(filename.as_str()) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Could not open {}: {}", filename, e);
                        success = false;
                        continue;
                    }
                };
                run_file_simple(&mut file);
            }
        }
    } else {
        let mut line_number = 1;
        for filename in filenames {
            if filename == "-" {
                let mut stdin = io::stdin();
                run_file(&mut stdin, &mut line_number, &args);
            } else {
                let mut file = match File::open(filename.as_str()) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Could not open {}: {}", filename, e);
                        success = false;
                        continue;
                    }
                };
                run_file(&mut file, &mut line_number, &args);
            }
        }
    }

    if !success {
        process::exit(1);
    }
}

// If there are no options used, just dump the files with no processing.
fn run_file_simple<R: Read>(file: &mut R) -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut buf = [0u8;64];
    let mut num_read = file.read(&mut buf)?;
    while num_read > 0 {
        stdout.write(&buf[..num_read])?;
        num_read = file.read(&mut buf)?;
    }
    Ok(())
}

fn run_file<R: Read>(
    file: &mut R,
    initial_line_number: &mut i32,
    args: &Args
) -> io::Result<()> {
    let mut stdout = io::stdout();
    let reader = BufReader::new(file);
    let mut prev_line_blank = true;
    let mut line_number = *initial_line_number;
    for line_result in reader.lines() {
        let line_str = line_result?;
        let line = line_str.as_bytes();
        if line.len() == 0 {
            if !args.squeeze_blank || !prev_line_blank {
                print_line(&mut stdout, line, line_number, args)?;
                line_number += 1;
            }
            prev_line_blank = true;
        } else {
            print_line(&mut stdout, line, line_number, args)?;
            line_number += 1;
        }
        stdout.flush()?;
    }
    *initial_line_number = line_number;
    Ok(())
}

fn print_line<W: Write>(
    out_stream: &mut W,
    line: &[u8],
    line_number: i32,
    args: &Args
) -> io::Result<()> {
    if args.number && (line.len() > 0 || !args.number_nonblank) {
        print_leading_number(out_stream, line_number)?;
    }
    out_stream.write(line)?;
    if args.show_ends {
        out_stream.write(&[b'$'])?;
    }
    out_stream.write(&[b'\n'])?;
    Ok(())
}

fn print_leading_number<W: Write>(out_stream: &mut W, number: i32) -> io::Result<()> {
    write!(out_stream, "{:6}  ", number)
}

fn parse_args() -> (Args, Vec<String>) {
    let app = App::new("cat")
        .about("Concatenate files to stdout")
        .version("v0.1.0")
        .arg(
            Arg::with_name("show-all")
                .short("A")
                .long("show-all")
                .help("Equivalent to -vET.")
        ).arg(
            Arg::with_name("number-nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Number all nonempty output lines, starting with 1.")
        ).arg(
            Arg::with_name("show-all-but-tabs")
                .short("e")
                .help("Equivalent to -vE.")
        ).arg(
            Arg::with_name("show-ends")
                .short("E")
                .long("show-ends")
                .help("Display a '$' after the end of each line.")
        ).arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Number all output lines, starting with 1.")
                .long_help(
                    "Number all output lines, starting with 1. \
                    This option is ignored if -b is in effect."
                )
        ).arg(
            Arg::with_name("squeeze-blank")
                .short("s")
                .long("squeeze-blank")
                .help("Suppress repeated adjacent blank lines.")
                .long_help(
                    "Suppress repeated adjacent blank lines; \
                    output just one empty line instead of several."
                )
        ).arg(
            Arg::with_name("show-all-but-ends")
                .short("t")
                .help("Equivalent to -vT.")
        ).arg(
            Arg::with_name("show-tabs")
                .short("T")
                .long("show-tabs")
                .help("Display TAB characters as '^I'.")
        ).arg(
            Arg::with_name("show-nonprinting")
                .short("v")
                .long("show-nonprinting")
                .help(
                    "Display control characters except for LFD and TAB \
                    using '^' notation."
                )
        ).arg(
            Arg::with_name("filenames")
                .index(1)
                .value_name("FILE")
                .multiple(true)
        );
    let matches = app.get_matches();

    let number = matches.is_present("number") || matches.is_present("number-nonblank");
    let number_nonblank = matches.is_present("number-nonblank");
    let squeeze_blank = matches.is_present("squeeze-blank");
    let show_ends =
        matches.is_present("show-ends") ||
        matches.is_present("show-all") ||
        matches.is_present("show-all-but-tabs");
    let show_tabs =
        matches.is_present("show-tabs") ||
        matches.is_present("show-all") ||
        matches.is_present("show-all-but-ends");
    let show_nonprinting =
        matches.is_present("show-nonprinting") ||
        matches.is_present("show-all");
    let filenames = match matches.values_of("filenames") {
        Some(i) => i.map(|s| s.to_string()).collect(),
        None => Vec::new(),
    };

    let args = Args {
        number,
        number_nonblank,
        squeeze_blank,
        show_ends,
        show_tabs,
        show_nonprinting,
    };
    (args, filenames)
}
