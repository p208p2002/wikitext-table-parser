use crate::tokenizer::CellTextSpecialTokens;
use crate::tokenizer::TableSpecialTokens;
use crate::tokenizer::Tokenizer;
use std::str::FromStr;

// https://en.wikiversity.org/wiki/Help:Wikitext_quick_reference

#[derive(Debug, Clone, Copy)]
pub enum State {
    Idle,
    ReadTable,
    ReadTableCaption,
    ReadCol,
    ReadRow,
}

#[derive(Debug, Clone)]
pub enum Event {
    TableStart,
    TableStyle(String),
    TableEnd,
    ColStart(CellType),
    ColStyle(String),
    ColEnd(String),
    TableCaptionStart,
    TableCaption(String),
    RowStart,
    RowStyle(String),
    RowEnd,
}

#[derive(Debug, Clone)]
pub enum CellType {
    HeaderCell,
    DataCell,
}

#[derive(Debug)]
pub struct WikitextTableParser {
    state: State,
    event_log_queue: Vec<Event>,
    tokens: Vec<String>,
    text_buffer: String,
    table_tokenizer: Tokenizer,
    cell_tokenizer: Tokenizer,
    clean_cell_text: bool,
}

impl Iterator for WikitextTableParser {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        while self.tokens.len() > 0 {
            self.step();
        }

        if self.event_log_queue.len() > 0 {
            let event = self.event_log_queue.remove(0);
            return Some(event);
        }
        return None;
    }
}

impl WikitextTableParser {
    pub fn new(
        table_tokenizer: Tokenizer,
        cell_tokenizer: Tokenizer,
        wikitext_table: &str,
        clean_cell_text: bool,
    ) -> Self {
        // add `\n` at start to match `\n{|`, even it is at the first of context.
        let text_for_parse = String::from("\n") + wikitext_table;
        let parser = WikitextTableParser {
            state: State::Idle,
            tokens: table_tokenizer.tokenize(&text_for_parse),
            event_log_queue: Vec::new(),
            text_buffer: String::from(""),
            table_tokenizer: table_tokenizer,
            cell_tokenizer: cell_tokenizer,
            clean_cell_text: clean_cell_text,
        };

        // println!("{:?}",parser.tokens);
        return parser;
    }

    fn append_to_text_buffer(&mut self, s: &str) {
        let token = TableSpecialTokens::from_str(s);
        match token {
            Ok(_) => {
                // do nothing if is a special token
            }
            Err(_) => {
                self.text_buffer += s;
            }
        }
    }

    fn clear_text_buffer(&mut self) {
        self.text_buffer = String::from("")
    }

    fn split_cell_style_and_text(&self, cell_text: String) -> Vec<String> {
        let cell_tokens = self.cell_tokenizer.tokenize(&cell_text);
        let mut style = String::new();
        let mut cell_text = String::new();
        let mut temp = String::new();
        let mut already_match_style_end = false;
        let mut in_closure = false;
        for token in cell_tokens {
            match CellTextSpecialTokens::from_str(token.as_str()) {
                Ok(cell_text_sp_token) => match cell_text_sp_token {
                    CellTextSpecialTokens::Sep => {
                        if !in_closure {
                            already_match_style_end = true;
                        }
                    }
                    CellTextSpecialTokens::LinkStart => in_closure = true,
                    CellTextSpecialTokens::LinkEnd => in_closure = false,
                    CellTextSpecialTokens::TemplateStart => in_closure = true,
                    CellTextSpecialTokens::TemplateEnd => in_closure = false,

                    _ => {}
                },
                Err(_) => {
                }
            }
            temp += &token;
            if already_match_style_end && style.len() == 0{
                style = temp.clone();
                temp = String::new();
            }
        }

        cell_text = temp;

        return vec![style,cell_text];
    }

    fn get_text_buffer_data(&self) -> String {
        let cell_raw_text = self.text_buffer.clone().trim().to_string();
        let split_texts = self.split_cell_style_and_text(cell_raw_text);
        let style: String = split_texts[0].clone();
        let cell_text = split_texts[1].clone();
        // println!("{:?}",style);
        return cell_text
    }

    fn get_style_text_buffer_data(&self) -> String {
        let cell_raw_text = self.text_buffer.clone().trim().to_string();
        let split_texts = self.split_cell_style_and_text(cell_raw_text);
        let style = split_texts[0].clone();
        let cell_text = split_texts[1].clone();
        // println!("{:?}",style);
        return style
    }

    fn step(&mut self) {
        let token = self.tokens.remove(0);
        match self.state {
            State::Idle => {
                if &token == TableSpecialTokens::TableStart.as_ref() {
                    self.transition(Event::TableStart)
                }
            }
            State::ReadTable => {
                self.append_to_text_buffer(&token);
                if &token == TableSpecialTokens::TableCaption.as_ref() {
                    self.transition(Event::TableStyle(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::TableCaptionStart);
                } else if &token == TableSpecialTokens::TableRow.as_ref() {
                    self.transition(Event::TableStyle(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                }
                // end of table
                else if &token == TableSpecialTokens::TableEnd.as_ref() {
                    self.transition(Event::TableEnd);
                }
            }

            State::ReadTableCaption => {
                self.append_to_text_buffer(&token);
                if &token == TableSpecialTokens::TableRow.as_ref() {
                    self.transition(Event::TableCaption(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                }
                // match ! after the caption, this type will not have a row style
                // and should turn in to read col state
                else if &token == TableSpecialTokens::TableHeaderCell.as_ref() {
                    self.transition(Event::TableCaption(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    // even we do not read the `|-` (row start token)
                    // we still send the read row event,
                    // after that send a col start event (due to we match `!`)
                    self.transition(Event::RowStart);
                    self.transition(Event::RowStyle(String::from("")));
                    self.transition(Event::ColStart(CellType::HeaderCell));
                }
            }

            State::ReadRow => {
                self.append_to_text_buffer(&token);
                if &token == TableSpecialTokens::TableDataCell.as_ref()
                    || &token == TableSpecialTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::RowStyle(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::ColStart(CellType::DataCell));
                } else if &token == TableSpecialTokens::TableHeaderCell.as_ref()
                    || &token == TableSpecialTokens::TableHeaderCell2.as_ref()
                {
                    self.transition(Event::RowStyle(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::ColStart(CellType::HeaderCell));
                } else if &token == TableSpecialTokens::TableEnd.as_ref() {
                    self.transition(Event::RowEnd);
                    self.transition(Event::TableEnd);
                    self.clear_text_buffer();
                }
            }

            State::ReadCol => {
                self.append_to_text_buffer(&token);

                // match \n| or \n||
                if &token == TableSpecialTokens::TableDataCell.as_ref()
                    || &token == TableSpecialTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::ColStyle(
                        self.get_style_text_buffer_data()
                    ));
                    self.transition(Event::ColEnd(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                }
                // match \n! or \n!!
                else if &token == TableSpecialTokens::TableHeaderCell.as_ref()
                    || &token == TableSpecialTokens::TableHeaderCell2.as_ref()
                {
                    self.transition(Event::ColStyle(
                        self.get_style_text_buffer_data()
                    ));
                    self.transition(Event::ColEnd(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                } else if &token == TableSpecialTokens::TableRow.as_ref() {
                    self.transition(Event::ColStyle(
                        self.get_style_text_buffer_data()
                    ));
                    self.transition(Event::ColEnd(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::RowEnd);
                    self.transition(Event::RowStart);
                } else if &token == TableSpecialTokens::TableEnd.as_ref() {
                    self.transition(Event::ColStyle(
                        self.get_style_text_buffer_data()
                    ));
                    self.transition(Event::ColEnd(
                        self.get_text_buffer_data(),
                    ));
                    self.clear_text_buffer();
                    self.transition(Event::RowEnd);
                    self.transition(Event::TableEnd);
                }
            }
        }
    }

    fn transition(&mut self, event: Event) {
        // println!(" -> {:?},{:?}", self.state, event);
        self.event_log_queue.push(event.clone());
        match (self.state, event) {
            // State::Idle
            (State::Idle, Event::TableStart) => self.state = State::ReadTable,

            // State::ReadTableCaption
            (State::ReadTableCaption, Event::TableCaption(_)) => self.state = State::ReadTable,

            // State::ReadTable
            (State::ReadTable, Event::TableCaptionStart) => self.state = State::ReadTableCaption,
            (State::ReadTable, Event::TableEnd) => {
                self.state = State::Idle;
            }
            (State::ReadTable, Event::RowStart) => self.state = State::ReadRow,

            // State::ReadRow
            (State::ReadRow, Event::ColStart(_)) => self.state = State::ReadCol,

            // State::ReadCol
            (State::ReadCol, Event::ColStyle(_)) => {}
            (State::ReadCol, Event::ColEnd(_)) => self.state = State::ReadCol,
            (State::ReadCol, Event::RowStart) => self.state = State::ReadRow,

            // Else
            (_, _) => {}
        }
    }
}
