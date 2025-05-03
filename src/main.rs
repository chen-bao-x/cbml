mod lexer;
mod parser;

fn main() {
    tests::test_parser();
}

// #[cfg(test)]
mod tests {

    use crate::{dp, lexer::tokenizer};

    // #[test]
    pub fn test_parser() {
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/3_enum.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/4_number.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/5_string.cbml");

        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/6_optinal.cbml");
    }

    fn asdfasdfsdf(path: &str) {
        use crate::parser::cbml_parser::CbmlParser;

        use std::fs::read_to_string;
        let code = read_to_string(path).unwrap();

        let tokens = tokenizer(&code)
            .map_err(|e| {
                println!("{}", e);
                return e;
            })
            .unwrap();

        dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(&tokens);
        let re = parser.parse();
        match re {
            Ok(statements) => {
                statements.iter().for_each(|s| {
                    dp(format!("statement: {:?}", s));
                });
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    dp(format!("message: {:?}", s.message));
                    dp(format!("tok: {:?}", s.token));
                });
            }
        }
    }
}

/// 只会在 debug 模式下打印输出.
pub fn dp<T>(s: T)
where
    T: ToString,
{
    #[cfg(debug_assertions)]
    {
        println!("{}", s.to_string());
    }
}
