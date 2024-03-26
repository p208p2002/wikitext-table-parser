use crate::tokenizer;
use crate::{tokenizer::SpecailTokens, utils};
use regex::{self, Regex};

// https://en.wikiversity.org/wiki/Help:Wikitext_quick_reference

#[derive(Debug, Clone, Copy)]
pub enum State {
    Idle,
    ReadTable,
    ReadTableStyle,
    ReadTableTitle,
    ReadCol,
    ReadTemplate,
    ReadLink,
    ReadRow,
    ReadHtml,
}

#[derive(Debug, Clone)]
pub enum Event {
    TableStart,
    TableStyleStart,
    TableStyle(String),
    TableEnd,
    ColStart,
    ColStyle(String),
    Col(String),
    TableTitleStart,
    TableTitle(String),
    TemplateStart,
    Template(String),
    LinkStart,
    Link(String),
    RowStart,
    Row(String),
    HtmlStart,
    Html(String),
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
        let mut parser = WikitextTableParser {
            state: State::Idle,
            tokens: tokenizer.tokenize(wikitext_table),
            event_log_queue: Vec::new(),
            text_buffer: String::from(""),
        };
        return parser;
    }

    // fn get_buffer_string(&mut self) -> String {
    //     return self.char_buffer.clone().iter().collect();
    // }

    // fn push_buffer(&mut self, c: char) {
    //     self.char_buffer.push(c);
    //     self.step();
    // }

    fn append_to_text_buffer(&mut self, s: &str) {
        self.text_buffer += s;
    }

    fn clear_text_buffer(&mut self) {
        self.text_buffer = String::from("")
    }

    // fn clear_some_buffer(&mut self, remain: usize) {
    //     while self.char_buffer.len() > remain {
    //         self.char_buffer.remove(0);
    //     }
    // }

    fn step(&mut self) {
        let token = self.tokens.remove(0);
        match self.state {
            State::Idle => {
                if &token == SpecailTokens::TableStart.as_ref() {
                    self.transition(Event::TableStart)
                }
            }
            State::ReadTable => {
                if &token == SpecailTokens::TableCaption.as_ref() {
                    self.transition(Event::TableTitleStart);
                }

                else if &token ==SpecailTokens::TableRow.as_ref() {
                    self.transition(Event::RowStart);

                }

                // end of table
                else if &token == SpecailTokens::TableEnd.as_ref() {
                    self.transition(Event::TableEnd);
                }
            }
            State::ReadTableTitle => {
                self.append_to_text_buffer(&token);
                if &token == SpecailTokens::TableRow.as_ref() {
                    self.transition(Event::TableTitle(self.text_buffer.clone()));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                } else if &token == SpecailTokens::TableHeaderCell.as_ref() {
                    self.transition(Event::TableTitle(self.text_buffer.clone()));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                    self.transition(Event::Row(String::from("dummy row style")));
                    self.transition(Event::ColStart);
                }

            }

            State::ReadRow => {
                if &token == SpecailTokens::TableDataCell.as_ref()
                    || &token == SpecailTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::Row(String::from("dummy row style")));
                    self.transition(Event::ColStart);
                } else if &token == SpecailTokens::TableHeaderCell.as_ref()
                    || &token == SpecailTokens::TableHeaderCell2.as_ref()
                {
                    self.transition(Event::Row(String::from("dummy row style")));
                    self.transition(Event::ColStart);
                }
                else if &token == SpecailTokens::TableEnd.as_ref() {
                    self.transition(Event::TableEnd);
                }
            }

            State::ReadCol => {
                self.append_to_text_buffer(&token);

                // match | or ||
                if &token == SpecailTokens::TableDataCell.as_ref()
                    || &token == SpecailTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::Col(self.text_buffer.clone()));
                    self.clear_text_buffer();
                // match ! or !!
                } else if &token == SpecailTokens::TableHeaderCell.as_ref()
                    || &token == SpecailTokens::TableDataCell2.as_ref()
                {
                    self.transition(Event::Col(self.text_buffer.clone()));
                    self.clear_text_buffer();
                } else if &token == SpecailTokens::TableRow.as_ref() {
                    self.transition(Event::Col(self.text_buffer.clone()));
                    self.clear_text_buffer();
                    self.transition(Event::RowStart);
                } else if &token == SpecailTokens::TableEnd.as_ref() {
                    self.transition(Event::Col(self.text_buffer.clone()));
                    self.clear_text_buffer();
                    self.transition(Event::TableEnd);
                }
            }

            _ => {}
        }

        // State::ReadTableStyle => {
        //     if Regex::new(r"\n$").unwrap().is_match(&buffer_string) {
        //         let clean_col_text = utils::clean_col_text(&buffer_string);
        //         self.transition(Event::TableStyle(clean_col_text));
        //         self.clear_buffer();
        //     }
        // }

        // State::ReadTable => {
        //     // match | or || but not (|- or |+ or |`$blank`)}
        //     if Regex::new(r"(\|){1,2}[^-\+}]$|^\|[^-\+}]$")
        //         .unwrap()
        //         .is_match(&buffer_string)
        //     {
        //         self.transition(Event::ColStart);
        //         // the regex judge with 2 char (look behind),
        //         // so keep 1 char for other condition.
        //         self.clear_some_buffer(1);
        //     }
        //     // match ! or !!}
        //     else if Regex::new(r"!").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::ColStart);
        //     }
        //     // table title |+
        //     else if Regex::new(r"\|\+").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::TableTitleStart);
        //         self.clear_buffer();
        //     }
        //     // row sep |-
        //     else if Regex::new(r"\|\-").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::RowStart);
        //         self.clear_buffer();
        //     }
        //     // end of table
        //     else if Regex::new(r"\|\}").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::TableEnd);
        //         self.clear_buffer();
        //     }
        // }
        // State::ReadCol => {
        //     // match \n| (end of col)
        //     if Regex::new(r"\n\|$|\n\!$").unwrap().is_match(&buffer_string) {
        //         let clean_col_text = utils::clean_col_text(&buffer_string);
        //         self.transition(Event::Col(clean_col_text));
        //         self.clear_some_buffer(1);
        //     }
        //     // match ||
        //     else if Regex::new(r".\|\|$").unwrap().is_match(&buffer_string) {
        //         let clean_col_text = utils::clean_col_text(&buffer_string);
        //         self.transition(Event::Col(clean_col_text));
        //         self.clear_buffer();
        //         // match inline sep, should immediatley start
        //         self.transition(Event::ColStart);
        //     }
        //     // match !!
        //     else if Regex::new(r".\!\!$").unwrap().is_match(&buffer_string) {
        //         let clean_col_text = utils::clean_col_text(&buffer_string);
        //         self.transition(Event::Col(clean_col_text));
        //         self.clear_buffer();
        //         // match inline sep, should immediatley start
        //         self.transition(Event::ColStart);
        //     }
        //     // match {{ (a wiki template start)
        //     else if Regex::new(r"\{\{$").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::TemplateStart);
        //     }
        //     // match [[ (a link sytanx start)
        //     else if Regex::new(r"\[\[$").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::LinkStart);
        //     }
        //     // match `<col_style>|` in col
        //     else if Regex::new(r"[^\n]\|[^\|]$")
        //         .unwrap()
        //         .is_match(&buffer_string)
        //     {
        //         let clean_col_text = utils::clean_col_text(&buffer_string);
        //         self.transition(Event::ColStyle(clean_col_text));
        //         self.clear_some_buffer(1);
        //     }
        //     // match a start of html tag
        //     else if Regex::new(r"<[^b\/>][^>]*[^\/]>$")
        //         .unwrap()
        //         .is_match(&buffer_string)
        //     {
        //         self.transition(Event::HtmlStart);
        //     }
        // }
        // State::ReadTableTitle => {
        //     // \n
        //     if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::TableTitle(buffer_string));
        //         self.clear_buffer();
        //     }
        // }
        // State::ReadTemplate => {
        //     if Regex::new(r"\}\}$").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::Template(buffer_string));
        //     }
        // }
        // State::ReadLink => {
        //     if Regex::new(r"\]\]$").unwrap().is_match(&buffer_string) {
        //         self.transition(Event::Link(buffer_string));
        //     }
        // }
        // State::ReadRow => {
        //     if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
        //         let clean_col_text = utils::clean_col_text(&buffer_string);
        //         self.transition(Event::Row(clean_col_text));
        //         self.clear_buffer();
        //     }
        // }
        // State::ReadHtml => {
        //     if Regex::new(r"<\/\s*([a-zA-Z][^\s>]*)\s*>$")
        //         .unwrap()
        //         .is_match(&buffer_string)
        //     {
        //         self.transition(Event::Html(buffer_string));
        //     }
        // }
        // }
    }

    fn transition(&mut self, event: Event) {
        println!(" -> {:?},{:?}", self.state, event);
        self.event_log_queue.push(event.clone());
        // println!("\t\tSTATE: {:?} EVENT: {:?} BF: {:?}", self.state, event,self.char_buffer);
        match (self.state, event) {
            // State::Idle
            (State::Idle, Event::TableStart) => self.state = State::ReadTable,

            // State::ReadTableStyle
            (State::ReadTableStyle, Event::TableStyle(_)) => self.state = State::ReadTable,

            // State::ReadTableTitle
            (State::ReadTableTitle, Event::TableTitle(_)) => self.state = State::ReadTable,

            // State::ReadTable
            (State::ReadTable, Event::TableStyleStart) => self.state = State::ReadTableStyle,
            (State::ReadTable, Event::TableTitleStart) => self.state = State::ReadTableTitle,
            // (State::ReadTable, Event::ColStart) => self.state = State::ReadCol,
            (State::ReadTable, Event::TableEnd) => {
                self.state = State::Idle;
            }
            (State::ReadTable, Event::RowStart) => self.state = State::ReadRow,

            // State::ReadRow
            (State::ReadRow, Event::ColStart) => self.state = State::ReadCol,

            // State::ReadTemplate
            (State::ReadTemplate, Event::Template(_)) => {
                self.state = State::ReadCol;
            }

            // State::ReadCol
            (State::ReadCol, Event::HtmlStart) => self.state = State::ReadHtml,
            (State::ReadCol, Event::TemplateStart) => self.state = State::ReadTemplate,
            (State::ReadCol, Event::LinkStart) => self.state = State::ReadLink,
            (State::ReadCol, Event::ColStyle(_)) => {}
            (State::ReadCol, Event::Col(_)) => self.state = State::ReadCol,
            (State::ReadCol, Event::RowStart) => self.state = State::ReadRow,

            // State::ReadLink
            (State::ReadLink, Event::Link(_)) => self.state = State::ReadCol,

            //State::ReadHtml
            (State::ReadHtml, Event::Html(_)) => self.state = State::ReadCol,

            // Else
            (_, _) => {}
        }
    }
}
