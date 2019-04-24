use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::process;

use clap::{clap_app, ArgMatches};

fn main() -> io::Result<()> {
    let options = parse_args();
    let reader = make_multi_reader(&options.filenames)?;

    let mut line_number = options.start;
    let mut section = PageSection::Body;

    let body_delim = options.delims.repeat(2);
    let header_delim = options.delims.repeat(3);

    for line_result in reader.lines() {
        let line = line_result?;
        let mut section_change = false;

        // Identify section splitters and reset the line number if necessary
        if line == header_delim {
            section = PageSection::Header;
            section_change = true;
        } else if line == body_delim {
            section = PageSection::Body;
            section_change = true;
        } else if line == options.delims {
            section = PageSection::Footer;
            section_change = true;
        }

        if section_change {
            println!();
            if options.reset_each_section {
                line_number = options.start;
            }
        } else {
            let style = section.get_style(&options);
            if style.number_it(&line) {
                println!(
                    "{num:width$}{sep}{line}",
                    num = line_number,
                    line = line,
                    sep = options.separator,
                    width = options.width,
                );
                line_number += options.increment;
            } else {
                println!("\t{}", line);
            }
        }
    }

    Ok(())
}

/// Return a single reader which chains together all the given files. A
/// filename of `"-"` represents `stdin`.
fn make_multi_reader(filenames: &[String]) -> io::Result<Box<dyn BufRead>> {
    let mut reader: Box<dyn BufRead> = Box::new(io::empty());
    for filename in filenames {
        if filename == "-" {
            let file = BufReader::new(io::stdin());
            reader = Box::new(reader.chain(file));
        } else {
            let file = BufReader::new(File::open(filename)?);
            reader = Box::new(reader.chain(file));
        }
    }
    Ok(reader)
}

fn parse_args() -> Options {
    let app = clap_app!(nl =>
        (version: "0.1")
        (author: "Chris Neidhart")
        (about: "Write each FILE to standard output, with line numbers added.")
        (@arg FILE: !required +multiple "input file to use")
        (@arg body_style: -b --("body-numbering") [STYLE] "use STYLE for numbering body lines")
        (@arg delimiter: -d --("section-delimiter") [CC] "use CC for logical page delimiters")
        (@arg footer_style: -f --("footer-numbering") [STYLE] "use STYLE for numbering footer lines")
        (@arg header_style: -h --("header-numbering") [STYLE] "use STYLE for numbering header lines")
        (@arg increment: -i --("line-increment") [NUMBER] "line number increment at each line")
        (@arg starting_line_number: -v --("starting-line-number") [NUMBER] "first line number for each section")
        (@arg separator: -s --("number-separator") [STRING] "add STRING after line number")
        (@arg no_renumber: -p --("no-renumber") "do not reset line numbers for each section")
        (@arg width: -w --("number-width") [NUMBER] "use NUMBER columns for line numbers")
    );
    let matches = app.get_matches();

    let filenames = match matches.values_of("FILE") {
        Some(values) => values.map(str::to_string).collect(),
        None => vec!["-".to_string()],
    };

    let header_style = Style::from_args(&matches, PageSection::Header);
    let body_style = Style::from_args(&matches, PageSection::Body);
    let footer_style = Style::from_args(&matches, PageSection::Footer);

    let increment = number_arg(&matches, "increment", "line number increment").unwrap_or(1);
    let start = number_arg(&matches, "starting_line_number", "starting line number").unwrap_or(1);
    let width = number_arg(&matches, "width", "line number field width").unwrap_or(6);

    let separator = matches.value_of("separator").unwrap_or("\t").to_string();
    let reset_each_section = !matches.is_present("no_renumber");

    let delims = match matches.value_of("delimiter") {
        Some(input) if input.len() == 1 => {
            let mut s = input.to_string();
            s.push(':');
            s
        }
        Some(input) => input.to_string(),
        None => r"\:".to_string(),
    };

    Options {
        filenames,
        body_style,
        increment,
        header_style,
        footer_style,
        start,
        separator,
        reset_each_section,
        width,
        delims,
    }
}

fn number_arg(matches: &ArgMatches, id: &str, message: &str) -> Option<usize> {
    matches.value_of(id).map(|s| {
        s.parse().unwrap_or_else(|_| {
            eprintln!("nl: invalid {}: '{}'", message, s);
            process::exit(1);
        })
    })
}

struct Options {
    filenames: Vec<String>,
    body_style: Style,
    header_style: Style,
    footer_style: Style,
    start: usize,
    delims: String,
    increment: usize,
    // join_blanks: u8,
    reset_each_section: bool,
    separator: String,
    // start: u8,
    width: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum PageSection {
    Header,
    Body,
    Footer,
}

impl PageSection {
    fn default_style(&self) -> Style {
        match *self {
            PageSection::Body => Style::NonEmpty,
            _ => Style::None,
        }
    }

    fn get_style(&self, options: &Options) -> Style {
        match *self {
            PageSection::Header => options.header_style,
            PageSection::Body => options.body_style,
            PageSection::Footer => options.footer_style,
        }
    }
}

#[derive(Clone, Copy)]
enum Style {
    All,
    NonEmpty,
    None,
    // Regex,
}

impl Style {
    fn number_it(&self, line: &str) -> bool {
        match self {
            Style::All => true,
            Style::NonEmpty => !line.is_empty(),
            Style::None => false,
        }
    }

    fn from_args(matches: &ArgMatches, section: PageSection) -> Self {
        let match_id = match section {
            PageSection::Header => "header_style",
            PageSection::Body => "body_style",
            PageSection::Footer => "footer_style",
        };
        matches
            .value_of(match_id)
            .map(|s| Style::from_code(s).unwrap_or_else(|_| bad_style(s, section)))
            .unwrap_or(section.default_style())
    }

    fn from_code(arg: &str) -> Result<Self, ()> {
        match arg {
            "a" => Ok(Style::All),
            "t" => Ok(Style::NonEmpty),
            "n" => Ok(Style::None),
            _ => Err(()),
        }
    }
}

fn bad_style(input: &str, section: PageSection) -> ! {
    let section_name = match section {
        PageSection::Header => "header",
        PageSection::Body => "body",
        PageSection::Footer => "footer",
    };

    eprintln!("nl: invalid {} numbering style: '{}'", section_name, input);
    eprintln!("Try 'nl --help' for more information.");
    process::exit(1);
}
