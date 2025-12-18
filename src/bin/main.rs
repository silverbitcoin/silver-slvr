//! Slvr REPL and CLI tool

use silver_slvr::{Lexer, Parser, VERSION, LANGUAGE_NAME};
use std::io::{self, Write};

fn main() {
    println!("{} v{}", LANGUAGE_NAME, VERSION);
    println!("Type 'exit' to quit, 'help' for commands\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("slvr> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        if input == "help" {
            print_help();
            continue;
        }

        match execute_command(input) {
            Ok(output) => println!("{}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn execute_command(input: &str) -> Result<String, String> {
    // Tokenize
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    
    // Parse
    let mut parser = Parser::new(input).map_err(|e| e.to_string())?;
    let _program = parser.parse().map_err(|e| e.to_string())?;

    Ok(format!("Parsed {} tokens", tokens.len()))
}

fn print_help() {
    println!("Available commands:");
    println!("  exit, quit    - Exit the REPL");
    println!("  help          - Show this help message");
    println!("\nSlvr Language Features:");
    println!("  - Turing-incomplete smart contract language");
    println!("  - Database-focused operations");
    println!("  - Fuel metering for execution costs");
    println!("  - Type-safe with compile-time checking");
}
