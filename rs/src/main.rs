mod lexer;
mod types;

fn main() {
    let content = std::fs::read_to_string("script.tan").expect("Could not read file");
    let tokens: Vec<lexer::LexerToken> = lexer::tokenize(&content);

    for token in tokens {
        println!("{:?}", token);
    }
}
