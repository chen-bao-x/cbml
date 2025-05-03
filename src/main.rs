mod lexer;
mod parser;

fn main() {
    lexer::tokenizer("name = \"hello\"").unwrap();
    tests::test_parser();
}

mod tests {

    use crate::lexer::tokenizer;

    // #[test]
    pub fn test_parser() {
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml");
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml");

        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");
    }

    fn asdfasdfsdf(path: &str) {
        use crate::parser::cbml_parser::CbmlParser;

        use std::fs::read_to_string;
        let code = read_to_string(path).unwrap();

        let tokens = tokenizer(&code).unwrap();
        println!("tokens: {:?}", tokens);
        let mut parser = CbmlParser::new(&tokens);
        let re = parser.parse();
        match re {
            Ok(statements) => {
                statements.iter().for_each(|s| {
                    println!("statement: {:?}", s);
                });
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    println!("message: {:?}", s.message);
                    println!("tok: {:?}", s.token);
                });
            }
        }
    }
}
