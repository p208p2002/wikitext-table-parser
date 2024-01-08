use regex::{self, Regex};
use std::fs::File;
use std::io::Read;

// https://en.wikiversity.org/wiki/Help:Wikitext_quick_reference

#[derive(Debug, Clone, Copy)]
enum State {
    Idle,
    ReadTableTitle,
    ReadColText,
    ReadTableStyle,
    InsideTable,
    ReadTemplate,
    ReadLink,
    ReadRow
}

#[derive(Debug)]
enum Event {
    TableStart,
    TableEnd,
    ColStart,
    ColStyle(String),
    ColEnd(String),
    TableTitleStart,
    TableTitleEnd(String),
    TemplateStart,
    TemplateEnd(String),
    LinkStart,
    LinkEnd(String),
    RowStart,
    RowEnd(String)
}

#[derive(Debug)]
struct StateMachine {
    state: State,
    char_buffer: Vec<char>,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine {
            state: State::Idle,
            char_buffer: Vec::new(),
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

    fn clear_some_buffer(&mut self,remain:usize){
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
                    self.clear_buffer();
                }
            }
            State::ReadTableStyle => {
                self.transition(Event::ColStart);
                if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
                    self.transition(Event::ColEnd(buffer_string));
                    self.clear_buffer();
                }
            }
            State::InsideTable => {
                // match | or || but not (|- or |+ or |`$blank`)}
                if Regex::new(r"(\|){1,2}[^-\+\s}]$")
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
            State::ReadColText => {
                // match \n
                if Regex::new(r"\n$").unwrap().is_match(&buffer_string) {
                    let clean_col_text = Regex::new(r"^(!|\|)|\n$").unwrap().replace_all(&buffer_string, "").trim().to_string();
                    self.transition(Event::ColEnd(clean_col_text));
                    self.clear_buffer();
                }
                // match ||
                else if Regex::new(r"\|\|$").unwrap().is_match(&buffer_string) {
                    let clean_col_text = Regex::new(r"^(!|\|)|\|\|$").unwrap().replace_all(&buffer_string, "").trim().to_string();
                    self.transition(Event::ColEnd(clean_col_text));
                    // match inline sep, should immediatley start
                    self.transition(Event::ColStart);
                }
                // match !!
                else if Regex::new(r"!{2}$").unwrap().is_match(&buffer_string) {
                    let clean_col_text = Regex::new(r"^(!|\|)|\!\!$").unwrap().replace_all(&buffer_string, "").trim().to_string();
                    self.transition(Event::ColEnd(clean_col_text));
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
                else if Regex::new(r"[^\n]\|[^\|]$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::ColStyle(buffer_string));
                    self.clear_buffer();
                }
            }
            State::ReadTableTitle => {
                // \n
                if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TableTitleEnd(buffer_string));
                    self.clear_buffer();
                }
            }
            State::ReadTemplate => {
                if Regex::new(r"\}\}$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::TemplateEnd(buffer_string));
                }
            }
            State::ReadLink => {
                if Regex::new(r"\]\]$").unwrap().is_match(&buffer_string) {
                    self.transition(Event::LinkEnd(buffer_string));
                }
            }
            State::ReadRow =>{
                if Regex::new(r"\n").unwrap().is_match(&buffer_string) {
                    self.transition(Event::RowEnd(buffer_string));
                    self.clear_buffer();
                }
            }
        }
    }

    fn transition(&mut self, event: Event) {
        // println!("\t\tSTATE: {:?} EVENT: {:?}", self.state, event);
        match (self.state, event) {
            // State::Idle
            (State::Idle, Event::TableStart) => self.state = State::ReadTableStyle,

            // State::ReadTableStyle
            (State::ReadTableStyle, Event::ColEnd(text)) => {
                println!("table_style {}", text);
                self.state = State::InsideTable
            }

            // State::ReadTableTitle
            (State::ReadTableTitle, Event::TableTitleEnd(text)) => {
                println!("table title {}", text);
                self.state = State::InsideTable
            }

            // State::InsideTable
            (State::InsideTable, Event::TableTitleStart) => self.state = State::ReadTableTitle,
            (State::InsideTable, Event::ColStart) => self.state = State::ReadColText,
            (State::InsideTable, Event::TableEnd) => {
                self.state = State::Idle;
                println!("====== TABLE EOF ======")
            }
            (State::InsideTable, Event::RowStart) => {
                self.state = State::ReadRow
            }

            // State::ReadRow
            (State::ReadRow,Event::RowEnd(text)) => {
                self.state = State::InsideTable;
                println!("----- {:?} -----",text);
            }

            // State::ReadTemplate
            (State::ReadTemplate, Event::TemplateEnd(text)) => {
                self.state = State::ReadColText;
            }

            // State::ReadColText
            (State::ReadColText, Event::TemplateStart) => self.state = State::ReadTemplate,
            (State::ReadColText, Event::LinkStart) => self.state = State::ReadLink,
            (State::ReadColText, Event::ColStyle(text))=>{
                // print!("col_style {:?}#",text);
            }
            (State::ReadColText, Event::ColEnd(text)) => {
                // let col_text = text.clone().trim().to_string();
                println!("col_text {:?}#", text);
                self.state = State::InsideTable
            }

            // State::ReadLink
            (State::ReadLink, Event::LinkEnd(_)) => self.state = State::ReadColText,
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
        // println!("\t\t{:}", c);
        state_machine.push_buffer(c);
        let x = state_machine.get_buffer_string();
        // println!("State->{:?} {:?}",state_machine.state,x);
    }

}
