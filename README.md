# Wiki Markup Table Parser

A Wiki Markup table parser written in Rust.

### What is this project for ?
Wiki Markup is a special format used by wikipedia, most available wiki data or processing tool ignore the table data. This project implement a wikitext-table parser that help one to processing the table in wikitext or wiki-dump.

A table in wikitext should like:
```
{| class="wikitable"
|+ Caption text
|-
! Header text !! Header text !! Header text
|-
| Example || Example || Example
|-
| Example || Example || Example
|-
| Example || Example || Example
|}
```
> also see the reference of [wikitext table](https://en.wikiversity.org/wiki/Help:Wikitext_quick_reference) for more detail.

### Usage Example
```rust
use std::fs::File;
use std::io::Read;
use std::env;
use wikitext_table_parser::parser::{Event, WikitextTableParser};

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
            Event::TableStart => {
                println!("Table START!");
            }
            Event::TableStyle(table_style) => {
                println!("table style{:?}#", table_style);
            }
            Event::TableCaption(text) => {
                println!("table name{:?}#", text);
            }
            Event::RowStyle(row_style) => {
                println!("----- {:?} -----", row_style);
            }
            Event::ColStyle(col_style) => {
                print!("col style: {:?}# ", col_style);
            }
            Event::Col(text) => {
                println!("col: {:?}#", text);
            }
            Event::TableEnd => {
                println!("Table END!");
            }
            _ => {}
        }
    }
}
```