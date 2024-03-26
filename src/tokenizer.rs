use core::fmt;
use std::collections::HashMap;
use std::str::FromStr;
use strum_macros::EnumString;
use strum_macros::AsRefStr;

// Wiki wable special token markup definition 
// https://en.wikipedia.org/wiki/Help:Table#Basic_table_markup
#[derive(Debug, PartialEq, EnumString,AsRefStr)]
pub enum SpecailTokens {
    #[strum(serialize = "\n{|")]
    TableStart,

    #[strum(serialize = "\n|+")]
    TableCaption,

    #[strum(serialize = "\n|-")]
    TableRow,

    #[strum(serialize = "\n!")]
    TableHeaderCell,

    #[strum(serialize="!!")]
    TableHeaderCell2,

    #[strum(serialize = "\n|")]
    TableDataCell,

    #[strum(serialize="||")]
    TableDataCell2,

    #[strum(serialize = "\n|}")]
    TableEnd,

    #[strum(serialize = "<nowiki>")]
    NoWikiStart,

    #[strum(serialize = "</nowiki>")]
    NoWikiEnd,
}

#[derive(Debug, Clone)]
pub struct TokenParseTreeNode {
    val: char,
    children: HashMap<char, TokenParseTreeNode>,
}

pub struct Tokenizer {
    token_tree: TokenParseTreeNode,
}

impl fmt::Display for TokenParseTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut children_vals: Vec<char> = Vec::new();
        let mut out = String::new();
        out = out + format!("val: {}", self.val).as_str();
        for child in self.children.iter() {
            children_vals.push(child.0.clone());
        }

        write!(f, "{}\n  children{:?}", out, children_vals)
    }
}

impl Tokenizer {
    pub fn build() -> Self {
        let special_tokens = [
            SpecailTokens::TableStart.as_ref(),
            SpecailTokens::TableEnd.as_ref(),
            SpecailTokens::TableCaption.as_ref(),
            SpecailTokens::TableDataCell.as_ref(),
            SpecailTokens::TableDataCell2.as_ref(),
            SpecailTokens::TableRow.as_ref(),
            SpecailTokens::TableHeaderCell.as_ref(),
            SpecailTokens::TableHeaderCell2.as_ref(),
            SpecailTokens::NoWikiStart.as_ref(),
            SpecailTokens::NoWikiEnd.as_ref()
        ];

        let mut root_node = TokenParseTreeNode {
            val: '$', // a root's val is unused
            children: HashMap::new(),
        };

        for token in special_tokens {
            println!("Token:{}", token);

            let mut node = &mut root_node;
            for t_char in token.chars() {
                match node.clone().children.get(&t_char) {
                    Some(_) => {
                        // if a value is alirady in children
                        // forward the node to the child.
                        node = node.children.get_mut(&t_char).unwrap();
                        println!("\t exist:  {}", t_char);
                    }
                    None => {
                        // finally we reach the bottom of the tree branch,
                        // insert the value in it
                        node.children.insert(
                            t_char.clone(),
                            TokenParseTreeNode {
                                val: t_char.clone(),
                                children: HashMap::new(),
                            },
                        );
                        node = node.children.get_mut(&t_char).unwrap();
                        println!("\t insert: {}", t_char);
                    }
                }
            }
            // root_node = node;
        }
        // println!("----------");
        // println!("{:}", root_node);
        return Tokenizer {
            token_tree: root_node,
        };
    }

    pub fn tokenize(&self, raw_str: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();

        let mut node = &self.token_tree;

        let mut tmp: String = String::new();
        for t_char in raw_str.chars() {
            match node.clone().children.get(&t_char) {
                Some(_) => {
                    node = node.children.get(&t_char).unwrap();
                    tmp = tmp + &node.val.to_string();
                }
                None => {
                    if tmp != "" {
                        out.push(tmp.clone());
                    }
                    // start from head for nex loop
                    node = &self.token_tree;

                    // process t_char which is this step
                    match node.children.get(&t_char) {
                        Some(_) => {
                            // if is a root's child
                            // forword to child
                            tmp = String::from(&t_char.to_string());
                            node = &self.token_tree.children.get(&t_char).unwrap();
                        }
                        None => {
                            // else, clean tmp and push the data to out
                            tmp = String::new();
                            out.push(String::from(&t_char.to_string()));
                        }
                    }
                }
            }
        }

        if tmp != "" {
            out.push(tmp);
        }

        // println!("{:?}", out);
        return out;
    }
}
