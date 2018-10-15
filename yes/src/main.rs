use std::env;
use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStrExt;

fn main() {
    let mut args = env::args_os();
    // discard argv[0]
    args.next();
    let mut out_line = OsString::new();
    match args.next() {
        Some(a) => {
            out_line.push(a);
            for arg in args {
                out_line.push(" ");
                out_lines.push(arg);
            };
        },
        None => {
            out_line.push("y");
        },
    }
    out_line.push("\n");
    let stdout = io::stdout();
    loop {
        stdout.write(out_line.as_bytes());
    }
}
