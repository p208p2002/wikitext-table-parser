use std::str::FromStr;

use crate::tokenizer;
use crate::tokenizer::SpecailTokens;
// use regex::{self, Regex};

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
    ColStart,
    ColStyle(String),
    Col(String),
    TableCaptionStart,
    TableCaption(String),
    RowStart,
    RowStyle(String),
}

#[derive(Debug)]
pub struct WikitextTableParser {
    state: State,
    event_log_queue: Vec<Event>,
    tokens: Vec<String>,
    text_buffer: String,
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
    pub fn new(wikitext_table: &str) -> Self {
        let tokenizer = tokenizer::Tokenizer::build();
        let parser = WikitextTableParser {
            state: State::Idle,
            tokens: tokenizer.tokenize(wikitext_table),
            event_log_queue: Vec::new(),
            text_buffer: String::from(""),
        };

        // println!("{:?}",parser.tokens);
        return parser;
    }

    fn append_to_text_buffer(&mut self, s: &str) {
        let token = SpecailTokens::from_str(s);
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

    fn get_text_buffer_data(&self) -> String {
        return self.text_buffer.clone().trim().to_string();
    }

    fn step(&mut self) {
        let token = self.tokens.remove(0);
        match self.state {
            State::Idle => {
                if &token == SpecailTokens::TableStart.as_ref() {
                    self.transition(Event::TableStart)
                }
            }
            State::ReadTable => {
                self.append_to_text_buffer(&token);
                if &token == SpecailTokens::TableCaption.as_ref() {
                    self.transition(Event::TableStyle(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    self.transition(Event::TableCaptionStart);
                } else if &token == SpecailTokens::TableRow.as_ref() {
                    self.transition(Event::TableStyle(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                }
                // end of table
                else if &token == SpecailTokens::TableEnd.as_ref() {
                    self.transition(Event::TableEnd);
                }
            }

            State::ReadTableCaption => {
                self.append_to_text_buffer(&token);
                if &token == SpecailTokens::TableRow.as_ref() {
                    self.transition(Event::TableCaption(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                }
                // match ! after the caption, this type will not have a row style
                // and should turn in to read col state
                else if &token == SpecailTokens::TableHeaderCell.as_ref() {
                    self.transition(Event::TableCaption(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    // even we do not read the `|-` (row start token)
                    // we still send the read row event,
                    // after that send a col start event (due to we match `!`)
                    self.transition(Event::RowStart);
                    self.transition(Event::RowStyle(String::from("")));
                    self.transition(Event::ColStart);
                }
            }

            State::ReadRow => {
                self.append_to_text_buffer(&token);
                if &token == SpecailTokens::TableDataCell.as_ref()
                    || &token == SpecailTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::RowStyle(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    self.transition(Event::ColStart);
                } else if &token == SpecailTokens::TableHeaderCell.as_ref()
                    || &token == SpecailTokens::TableHeaderCell2.as_ref()
                {
                    self.transition(Event::RowStyle(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    self.transition(Event::ColStart);
                } else if &token == SpecailTokens::TableEnd.as_ref() {
                    self.transition(Event::TableEnd);
                    self.clear_text_buffer();
                }
            }

            State::ReadCol => {
                self.append_to_text_buffer(&token);

                // match | or ||
                if &token == SpecailTokens::TableDataCell.as_ref()
                    || &token == SpecailTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::Col(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                // match ! or !!
                } else if &token == SpecailTokens::TableHeaderCell.as_ref()
                    || &token == SpecailTokens::TableHeaderCell2.as_ref()
                {
                    self.transition(Event::Col(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                } else if &token == SpecailTokens::TableRow.as_ref() {
                    self.transition(Event::Col(self.get_text_buffer_data()));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                } else if &token == SpecailTokens::TableEnd.as_ref() {
                    self.transition(Event::Col(self.get_text_buffer_data()));
                    self.clear_text_buffer();
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
            (State::ReadRow, Event::ColStart) => self.state = State::ReadCol,

            // State::ReadCol
            (State::ReadCol, Event::ColStyle(_)) => {}
            (State::ReadCol, Event::Col(_)) => self.state = State::ReadCol,
            (State::ReadCol, Event::RowStart) => self.state = State::ReadRow,

            // Else
            (_, _) => {}
        }
    }
}
