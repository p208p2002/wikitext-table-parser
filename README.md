# WikiText Table Parser

A WikiText table parser written in Rust.

### What is this project for ?
WikiText is a special format used by wikipedia, most available wiki data or processing tool ignore the table data. This project implement a table parser that help one to processing the table in wikitext or wiki-dump.

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

### Usage
#### Rust
##### Installation
```toml
[dependencies]
wikitext_table_parser = "0.2.4"
```
```rust
use std::env;
use std::fs::File;
use std::io::Read;
use wikitext_table_parser::parser::{Event, WikitextTableParser};
use wikitext_table_parser::tokenizer::{
    get_all_cell_text_special_tokens, get_all_table_special_tokens, Tokenizer,
};

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
    let table_tokenizer = Tokenizer::build(get_all_table_special_tokens());
    let cell_tokenizer = Tokenizer::build(get_all_cell_text_special_tokens());
    let wikitext_table_parser =
        WikitextTableParser::new(table_tokenizer, cell_tokenizer, &content, true);
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
                print!("col style: {:?} -- ", col_style);
            }
            Event::ColEnd(text) => {
                println!("col data: {:?}", text);
            }
            Event::TableEnd => {
                println!("Table END!");
            }
            _ => {}
        }
    }
}

```

#### Python
```python
import sys
from wikitext_table_parser import (
    WikitextTableParser,
    Tokenizer,
    Event,
    get_all_table_special_tokens,
    get_all_cell_text_special_tokens
)

table_tokens = get_all_table_special_tokens()
cell_tokens = get_all_cell_text_special_tokens()

table_tokenizer = Tokenizer(table_tokens)
cell_tokenizer = Tokenizer(cell_tokens)

test_case = open(sys.argv[-1]).read()

parser = WikitextTableParser(table_tokenizer, cell_tokenizer, test_case, True)
print(parser.tokens)

while (len(parser.tokens) > 0):
    parser.step()

for event in parser.event_log_queue:
    if isinstance(event, Event.TableStart):
        pass
    elif isinstance(event, Event.TableStyle):
        print("table style:", event.text)
    elif isinstance(event, Event.TableEnd):
        pass
    elif isinstance(event, Event.ColStart):
        print("col type:", event.cell_type)
    elif isinstance(event, Event.ColStyle):
        print("col style:", event.text)
    elif isinstance(event, Event.ColEnd):
        print("col data:", event.text)
    elif isinstance(event, Event.TableCaptionStart):
        pass
    elif isinstance(event, Event.TableCaption):
        print("table caption:", event.text)
    elif isinstance(event, Event.RowStart):
        pass
    elif isinstance(event, Event.RowStyle):
        print("row style:", event.text)
    elif isinstance(event, Event.RowEnd):
        print("-"*20)
    else:
        raise NotImplementedError(event)
```