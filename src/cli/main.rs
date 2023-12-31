use std::io::{BufRead, BufReader, stdin, Stdin, stdout, Stdout, Write};
use std::process::exit;
use headache::compiler::compile;
use headache::error::{Error, ParserError};
#[cfg(target_arch="x86_64")]
use headache::executor::Executor;
use crate::cli::{CLIError, get_mode, Mode};

mod cli;

/// Main function for the Headache Brainfuck interpreter program.
fn main() -> Result<(), Error> {
    // Determine the mode in which the program should run based on command line arguments.
    let mode = match get_mode() {
        Ok(mode) => mode,
        Err(err) => {
            match err {
                CLIError::IO(io) => { eprintln!("Cannot read the script {}", io) }
                CLIError::Cli(err) => { eprintln!("{err}") }
            }
            exit(1)
        }
    };

    let mut executor = Executor::default();

    // Execute the program based on the determined mode.
    match mode {
        Mode::Executor(source) => {
            #[cfg(target_arch="x86_64")]
            {
                let (mut stdin, mut stdout) = (stdin(), stdout());
                match compile(&source, &mut stdin, &mut stdout) {
                    Ok(exe) => exe.run()?,
                    Err(err) => {
                        match err {
                            Error::CompileError(_) =>{ executor.execute(&source)? }
                            _ => {return Err(err);}
                        }
                    },
                }
            }
            // Parse and execute a Brainfuck script from a file.
            #[cfg(not(target_arch="x86_64"))]
            executor.execute(&source)?
        }
        Mode::Interpreted => {
            interpreter(&mut executor)?
        }
    }
    Ok(())
}

fn interpreter(executor: &mut Executor<Stdin, Stdout>) -> Result<(), Error> {
    // Run the program in real-time interpreter mode.
    let mut buffer = String::new();
    println!("Write exit to finish the interpreter");
    loop {
        if buffer.is_empty() {
            print!(">")
        } else {
            print!("==>")
        }
        stdout().flush().unwrap();
        let mut reader = BufReader::new(stdin());
        reader.read_line(&mut buffer).unwrap();
        if buffer.contains("exit") {
            exit(0)
        }
        match executor.execute(&buffer) {
            Ok(_) => {},
            Err(err) => match err{
                Error::ParseError(err) => match err {
                    ParserError::IncompleteLoop => {continue;}
                    ParserError::UnexpectedToken => {
                        eprintln!("Error: Cannot close ']' without first open '[' it")
                    }
                }
                Error::RuntimeError(_) => {return Err(err)},
                #[cfg(target_arch="x86_64")]
                Error::CompileError(_) => {return Err(err);}
            }
        }
        buffer.clear()
    }
}