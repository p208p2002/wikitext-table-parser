pub mod parser;
pub mod utils;
pub mod tokenizer;

#[cfg(test)]
mod test_tokenizer {
    use crate::tokenizer;

    // #[test]
    // fn build_tokenizer() {
    //     tokenizer::StreamTokenizer::build();
    // }

    #[test]
    fn tokenize(){
        let raw_string = String::from("{|123|||}");
        let tokenizer = tokenizer::StreamTokenizer::build();
        tokenizer.tokenize(&raw_string);
    }
}