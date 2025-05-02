mod lexer;
mod parser;
fn main() {

    lexer::tokenizer("let a = 1;").unwrap();
}

// fn sadfdasf(path: &str) {
//     let sadf = std::fs::read_to_string(path).unwrap();
//     let a = token::Lexer::new(&sadf).tokenize();
//     match a {
//         Ok(tokens) => {
//             for token in tokens {
//                 println!("{:?}", token.kind);
//             }
//         }
//         Err(e) => {
//             println!("Error: {}", e);
//         }
//     }
// }
