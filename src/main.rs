use std::io;
use std::io::Write;

mod lexing;
mod parsing;

fn main() {
    let mut buffer = String::with_capacity(512);
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        buffer.clear();
        print!("\u{03c8}\u{03c8}\u{03c8}\u{03c8}> ");

        if let Err(error) = stdout.flush() {
            return eprintln!("\nIO Error: {}", error);
        }

        if let Err(error) = stdin.read_line(&mut buffer) {
            return eprintln!("\nIO Error: {}", error);
        }

        if buffer.is_empty() {
            return;
        }
    }
}
