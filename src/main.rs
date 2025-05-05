mod lexer;
mod parser;
mod typecheck;

#[allow(dead_code)]
fn main() {
    tests::test_parser();

    // config
    //
    // useStmt | linecomment | blockComment | asignment
    //

    // typedef
    //
}

fn fasdf(count: usize, f: fn()) {
    use std::time::Instant;
    let start = Instant::now(); // 记录开始时间

    for _ in 0..count {
        f();
    }

    let duration = start.elapsed(); // 计算耗时

    println!("耗时：{:?}", duration);
}

// #[cfg(test)]
mod tests {

    use crate::{dp, fasdf, lexer::tokenizer, typecheck::typecheck};

    // #[test]
    pub fn test_parser() {
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/3_enum.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/4_number.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/5_string.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/6_optinal.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/7_struct-cbml");

        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/8_union.cbml");
    }

    #[test]
    fn testr_enum() {
        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");
    }

    #[test]
    fn testr_arr() {
        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");
    }

    #[test]
    fn test_1() {
        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml");
    }

    #[test]
    fn test_struct() {
        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/7_struct-cbml");
    }

    fn asdfasdfsdf(path: &str) {
        use std::fs::read_to_string;
        let code = read_to_string(path).unwrap();
        dsafdasfsadf(&code);
    }

    fn dsafdasfsadf(code: &str) {
        use crate::parser::cbml_parser::CbmlParser;

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
            Ok(ast) => {
                // ast.iter().for_each(|s| {
                //     dp(format!("statement: {:?}", s));
                // });

                // dp("start typecheck: ");

                let re = typecheck(ast);
                re.iter().for_each(|x| {
                    dp(format!("{:?}", x));
                });
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    dp(format!("message: {:?}", s.message));
                    dp(format!("tok: {:?}", s.token));
                });

                panic!();
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
