use std::env;
use std::fs::File;
use std::io::Read;
use wikitext_table_parser::parser::{Event as ParserEvent, WikitextTableParser};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = args[1].clone();

    // Attempt to open the file
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Error opening the file.");
            return;
        }
    };

    // Read the contents of the file into a String
    let mut content = String::new();
    if let Err(_) = file.read_to_string(&mut content) {
        eprintln!("Error reading the file into a string.");
        return;
    }

    let wikitext_table_parser = WikitextTableParser::new(&content);
    for event in wikitext_table_parser {
        match event {
            ParserEvent::TableStart => {
                println!("Table START!");
            }
            ParserEvent::TableStyleEnd(table_style) => {
                println!("table style{:?}#", table_style);
            }
            ParserEvent::TableCaptionEnd(text) => {
                println!("table name{:?}#", text);
            }
            ParserEvent::Row(row_style) => {
                println!("----- {:?} -----", row_style);
            }
            ParserEvent::ColStyle(col_style) => {
                print!("col style: {:?}# ", col_style);
            }
            ParserEvent::Col(text) => {
                println!("col: {:?}#", text);
            }
            ParserEvent::TableEnd => {
                println!("Table END!");
            }
            _ => {}
        }
    }
}
