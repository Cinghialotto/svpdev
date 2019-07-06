mod asm;
mod assembler;
mod tokenization;

#[macro_use]
extern crate clap;
use clap::App;

use assembler::assembly;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use tokenization::tokens;

pub struct Config {
    pub input_filename: String,
    pub output_filename: String,
    pub is_debug: bool,
}

impl Config {
    pub fn new_from_args() -> Result<Config, ()> {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches();

        match (matches.value_of("INPUT"), matches.value_of("OUTPUT")) {
            (Some(input), Some(output)) => {
                let is_debug = matches.occurrences_of("debug") > 0;
                Ok(Config {
                    input_filename: input.to_string(),
                    output_filename: output.to_string(),
                    is_debug,
                })
            }
            _ => match App::from_yaml(yaml).print_long_help() {
                _ => Err(()),
            },
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.input_filename)?;
    let tokens = tokens::tokenize(contents.as_str())?;

    let (symbol_table, equ_table) = assembly::extract_tables(&tokens);
    let opcodes = assembly::generate_opcodes(&tokens, &symbol_table, &equ_table, config.is_debug)?;

    let mut file = File::create(config.output_filename)?;
    file.write_all(&opcodes)?;

    if config.is_debug {
        print_debug_info(&tokens, &symbol_table, &equ_table, &opcodes);
    }

    Ok(())
}

pub fn print_table<'a>(table: &HashMap<&'a str, u16>, title: &str) {
    if table.len() > 0 {
        println!("**** {} ****", title);
        table.iter().for_each(|(k, v)| println!("{} -> {}", k, v))
    } else {
        println!("No elements found for {}", title);
    }
}

pub fn print_debug_info<'a>(
    tokens: &Vec<tokens::Token>,
    symbol_table: &HashMap<&'a str, u16>,
    equ_table: &HashMap<&'a str, u16>,
    opcodes: &Vec<u8>,
) {
    if tokens.len() > 0 {
        println!("\nFound tokens:");
        for token in tokens {
            println!("{:?}", *token);
        }
    } else {
        println!("\nNo tokens found!");
    }

    println!("");
    print_table(&symbol_table, "Symbol table");
    println!("");
    print_table(&equ_table, "Constants table");

    println!("\nGenerated opcodes:");
    println!("{:02X?}", opcodes);
}
