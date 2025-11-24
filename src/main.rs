mod lexer;
mod operation;
mod parser;
mod token;
mod vm;

use std::io::{self, Write};

fn main() -> io::Result<()> {
    repl()
}



fn repl() -> io::Result<()> {
    loop {
        print!(">> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        
        let bytes = io::stdin().read_line(&mut input)?;
        if bytes == 0 {
            println!("\nExiting...");
            break;
        }

        if input.trim() == "q" {
            break;
        } else if input.trim().is_empty() {
            continue;
        }

        let mut lexer = lexer::Lexer::from_str(input.as_str());
        let tokens = lexer.scan();

        match tokens {
            Err(e) => eprintln!("{}", e),
            Ok(tokens) => {

                let mut parser = parser::Parser::new(tokens);
                let operations = parser.parse();
                
                match operations {
                    Err(e) => eprintln!("{}", e),
                    Ok(operations) => {

                        let result = vm::interpret(operations);
                        match result {
                            Ok(output ) => println!("{}", output),
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                }
            } 
        }
    }
    Ok(())
}
