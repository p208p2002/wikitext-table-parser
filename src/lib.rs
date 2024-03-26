pub mod parser;
pub mod tokenizer;
pub mod utils;

#[cfg(test)]
mod test_tokenizer {
    use crate::tokenizer;

    #[test]
    fn tokenize() {
        let raw_string = String::from("\n{|123||\n|}<><nowiki>");
        let expect_result = Vec::from(["\n{|", "1", "2", "3", "||", "\n|}","<",">","<nowiki>"]);
        let tokenizer = tokenizer::Tokenizer::build();
        let out = tokenizer.tokenize(&raw_string);
        assert_eq!(out.join(" / "), expect_result.join(" / "));
    }
}

#[cfg(test)]
mod test_parser {
    use crate::parser::{Event as ParserEvent, WikitextTableParser};
    use std::fs::File;
    use std::io::Read;

    fn test_parse_struct_table(path: String, expect_rows: i32, expect_cols: i32) {
        /* Test a wiki text table that has expect number of rows and cols */

        // read table from text
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                return;
            }
        };
        let mut content: String = String::new();
        if let Err(_) = file.read_to_string(&mut content) {
            return;
        }

        //
        let mut count_rows = 0;
        let mut count_cols = 0;
        let mut count_table_start = 0;
        let mut count_table_end = 0;

        let wikitext_table_parser = WikitextTableParser::new(&content);
        for event in wikitext_table_parser {
            match event {
              
                ParserEvent::Row(row_style) => {
                    if count_rows > 0 {
                        // do not work just after parse the first row, which is a table headr.
                        assert_eq!(expect_cols, count_cols);
                    }
                    count_rows += 1;
                    count_cols = 0;
                    println!("----- {:?} -----", row_style);
                }
                ParserEvent::Col(text) => {
                    count_cols += 1;
                    println!("col: {:?}#", text);
                }
                ParserEvent::TableStart =>{
                    count_table_start += 1
                }
                ParserEvent::TableEnd =>{
                    count_table_end += 1
                }
                _ => {}
            }
        }
        assert_eq!(expect_rows, count_rows);
        assert_eq!(1,count_table_start);
        assert_eq!(1,count_table_end)
    }

    #[test]
    fn test_parse_struct_table_1() {
        test_parse_struct_table(String::from("wikitext_tables/1.txt"), 11, 2)
    }

    #[test]
    fn test_parse_struct_table_2() {
        test_parse_struct_table(String::from("wikitext_tables/2.txt"), 5, 5)
    }

    #[test]
    fn test_parse_struct_table_3() {
        test_parse_struct_table(String::from("wikitext_tables/3.txt"), 12, 5)
    }

    #[test]
    fn test_parse_struct_table_4() {
        test_parse_struct_table(String::from("wikitext_tables/4.txt"), 8, 5)
    }
}
