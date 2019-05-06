//! A data structure which takes in multiple filenames and then behaves as a
//! Reader which chains together each file. It also supports a special feature:
//! The filename `"-"` maps to `Stdin` instead of an actual file on the system.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read};

pub struct MultiReader {
    filenames: VecDeque<String>,
    current: Box<dyn Read>,
}

impl Read for MultiReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.current.read(buf)?;
        if n == 0 {
            match self.filenames.pop_front() {
                Some(filename) => {
                    self.current = open_file(&filename)?;
                    self.read(buf)
                }
                None => Ok(0),
            }
        } else {
            Ok(n)
        }
    }
}

fn open_file(filename: &str) -> io::Result<Box<dyn Read>> {
    if filename == "-" {
        Ok(Box::new(io::stdin()))
    } else {
        Ok(Box::new(File::open(filename)?))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
