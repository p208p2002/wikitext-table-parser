use regex::{self, Regex};
use std::fs::File;
use std::io::Read;

// https://en.wikiversity.org/wiki/Help:Wikitext_quick_reference

#[derive(Debug, Clone, Copy)]
enum State {
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
enum Event {
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
struct StateMachine {
    state: State,
    event_log_queue: Vec<Event>,
    char_buffer: Vec<char>,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine {
            state: State::Idle,
            char_buffer: Vec::new(),
            event_log_queue: Vec::new(),
        }
    }

    fn get_buffer_string(&mut self) -> String {
        return self.char_buffer.clone().iter().collect();
    }

    fn push_buffer(&mut self, c: char) {
        self.char_buffer.push(c);
        self.step();
    }

    fn clear_buffer(&mut self) {
        self.char_buffer.clear();
    }

    fn clear_some_buffer(&mut self, remain: usize) {
        while self.char_buffer.len() > remain {
            self.char_buffer.remove(0);
        }
    }

    fn step(&mut self) {
        let buffer_string = self.get_buffer_string();
        match self.state {
            State::Idle => {
                // match {|, the sign of table start
                if Regex::new(r"\{\|").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TableStart);
                    self.transition(Event::TableStyleStart);
                    self.clear_buffer();
                } else if self.char_buffer.len() > 2 {
                    self.char_buffer.remove(0);
                }
            }

            State::ReadTableStyle => {
                if Regex::new(r"\n$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TableStyle(buffer_string));
                    self.clear_buffer();
                }
            }

            State::ReadTable => {
                // match | or || but not (|- or |+ or |`$blank`)}
                if Regex::new(r"(\|){1,2}[^-\+}]$|^\|[^-\+}]$")
                    .unwrap()
                    .is_match(&buffer_string)
                {
                    self.transition(Event::ColStart);
                    // the regex judge with 2 char (look behind),
                    // so keep 1 char for other condition.
                    self.clear_some_buffer(1);
                }
                // match ! or !!}
                else if Regex::new(r"!").unwrap().is_match(&buffer_string) {
                    self.transition(Event::ColStart);
                }
                // table title |+
                else if Regex::new(r"\|\+").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TableTitleStart);
                    self.clear_buffer();
                }
                // row sep |-
                else if Regex::new(r"\|\-").unwrap().is_match(&buffer_string) {
                    self.transition(Event::RowStart);
                    self.clear_buffer();
                }
                // end of table
                else if Regex::new(r"\|\}").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TableEnd);
                    self.clear_buffer();
                }
            }
            State::ReadCol => {
                // match \n| (end of col)
                if Regex::new(r"\n\|$|\n\!$").unwrap().is_match(&buffer_string) {
                    // if Regex::new(r"(\|[^\|])|(\![^\!])").unwrap().is_match(&buffer_string) {
                    let clean_col_text = Regex::new(r"^(!|\|)|\n$")
                        .unwrap()
                        .replace_all(&buffer_string, "")
                        .trim()
                        .to_string();
                    self.transition(Event::Col(clean_col_text));
                    self.clear_some_buffer(1);
                }
                // match ||
                else if Regex::new(r".\|\|$").unwrap().is_match(&buffer_string) {
                    let clean_col_text = Regex::new(r"^(!|\|)|\|\|$")
                        .unwrap()
                        .replace_all(&buffer_string, "")
                        .trim()
                        .to_string();
                    self.transition(Event::Col(clean_col_text));
                    self.clear_buffer();
                    // match inline sep, should immediatley start
                    self.transition(Event::ColStart);
                }
                // match !!
                else if Regex::new(r".\!\!$").unwrap().is_match(&buffer_string) {
                    let clean_col_text = Regex::new(r"^(!|\|)|\!\!$")
                        .unwrap()
                        .replace_all(&buffer_string, "")
                        .trim()
                        .to_string();
                    self.transition(Event::Col(clean_col_text));
                    self.clear_buffer();
                    // match inline sep, should immediatley start
                    self.transition(Event::ColStart);
                }
                // match {{ (a wiki template start)
                else if Regex::new(r"\{\{$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TemplateStart);
                }
                // match [[ (a link sytanx start)
                else if Regex::new(r"\[\[$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::LinkStart);
                }
                // match `<col_style>|` in col
                else if Regex::new(r"[^\n]\|[^\|]$")
                    .unwrap()
                    .is_match(&buffer_string)
                {
                    self.transition(Event::ColStyle(buffer_string));
                    self.clear_some_buffer(1);
                }
                // match a start of html tag
                else if Regex::new(r"<[^b\/>][^>]*[^\/]>$")
                    .unwrap()
                    .is_match(&buffer_string)
                {
                    self.transition(Event::HtmlStart);
                }
            }
            State::ReadTableTitle => {
                // \n
                if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TableTitle(buffer_string));
                    self.clear_buffer();
                }
            }
            State::ReadTemplate => {
                if Regex::new(r"\}\}$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::Template(buffer_string));
                }
            }
            State::ReadLink => {
                if Regex::new(r"\]\]$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::Link(buffer_string));
                }
            }
            State::ReadRow => {
                if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
                    self.transition(Event::Row(buffer_string));
                    self.clear_buffer();
                }
            }
            State::ReadHtml => {
                if Regex::new(r"<\/\s*([a-zA-Z][^\s>]*)\s*>$")
                    .unwrap()
                    .is_match(&buffer_string)
                {
                    self.transition(Event::Html(buffer_string));
                }
            }
        }
    }

    fn transition(&mut self, event: Event) {
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
            (State::ReadTable, Event::ColStart) => self.state = State::ReadCol,
            (State::ReadTable, Event::TableEnd) => {
                self.state = State::Idle;
            }
            (State::ReadTable, Event::RowStart) => self.state = State::ReadRow,

            // State::ReadRow
            (State::ReadRow, Event::Row(_)) => {
                self.state = State::ReadTable;
            }

            // State::ReadTemplate
            (State::ReadTemplate, Event::Template(_)) => {
                self.state = State::ReadCol;
            }

            // State::ReadCol
            (State::ReadCol, Event::HtmlStart) => self.state = State::ReadHtml,
            (State::ReadCol, Event::TemplateStart) => self.state = State::ReadTemplate,
            (State::ReadCol, Event::LinkStart) => self.state = State::ReadLink,
            (State::ReadCol, Event::ColStyle(_)) => {}
            (State::ReadCol, Event::Col(_)) => self.state = State::ReadTable,

            // State::ReadLink
            (State::ReadLink, Event::Link(_)) => self.state = State::ReadCol,

            //State::ReadHtml
            (State::ReadHtml, Event::Html(_)) => self.state = State::ReadCol,

            // Else
            (_, _) => {}
        }
    }
}

fn main() {
    let file_path = "./wiki_table.txt";
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

    let mut state_machine = StateMachine::new();

    for c in content.chars() {
        state_machine.push_buffer(c);
        while state_machine.event_log_queue.len() > 0 {
            let event = state_machine.event_log_queue.remove(0);
            match event {
                Event::TableStart => {
                    println!("Table START!");
                }
                Event::TableStyle(table_style) => {
                    println!("table style{:?}#", table_style);
                }
                Event::TableTitle(text) => {
                    println!("table name{:?}#", text);
                }
                Event::Row(row_style) => {
                    println!("\n----- {:?} -----", row_style);
                }
                Event::ColStyle(col_style) => {
                    print!("*{:?}*", col_style);
                }
                Event::Col(text) => {
                    println!("{:?}#", text);
                }
                Event::TableEnd => {
                    println!("Table END!");
                }
                _ => {}
            }
        }
    }
}
