pub mod parser;
pub mod utils;
pub mod tokenizer;

#[cfg(test)]
mod test_tokenizer {
    use crate::tokenizer;
    
    #[test]
    fn tokenize(){
        let raw_string = String::from("{|123|||}");
        let expect_result = Vec::from(["{|","1","2","3","||","|}"]);
        let tokenizer = tokenizer::Tokenizer::build();
        let out = tokenizer.tokenize(&raw_string);
        assert_eq!(out.join(" / "),expect_result.join(" / "));
    }
}