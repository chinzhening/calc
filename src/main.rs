mod lexer;
mod operation;
mod parser;
mod token;
mod vm;

use std::io::{self, Write};

fn main() -> io::Result<()> {
    welcome();
    repl()
}

fn welcome() {
    println!("Welcome to Calc!\n");
    println!("Press 'q' to quit.");
    println!("Type '--mode=radian' to use radians, and '--mode=degree' to use degrees.");
    println!("");
}

fn repl() -> io::Result<()> {
    let mut vm = vm::VirtualMachine::new();

    loop {
        print!(">> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        
        let bytes = io::stdin().read_line(&mut input)?;
        if bytes == 0 {
            println!("\nExiting...");
            break;
        }

        match input.trim() {
            "q" | "exit" => break,
            "--mode=radian" => {vm.use_radians=true; continue},
            "--mode=degree" => {vm.use_radians=false; continue},
            "" => continue,
            _ => {},
        }

        let tokens = lexer::scan(input);

        match tokens {
            Err(e) => eprintln!("{}", e),
            Ok(tokens) => {
                let operations = parser::parse(tokens);
                
                match operations {
                    Err(e) => eprintln!("{}", e),
                    Ok(operations) => {

                        let result = vm.interpret(&operations);
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
