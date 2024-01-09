use regex::Regex;

pub fn clean_col_text(text: &str) -> String {
    let mut clean_text = text.trim().to_string();
    clean_text = Regex::new(r"^(\||\!)+|(\||\!)+$")
        .unwrap()
        .replace_all(&clean_text, "")
        .trim()
        .to_string();
    return  clean_text;
}
