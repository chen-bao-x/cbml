mod lexer;
mod parser;
mod source_map;
mod typecheck;

// fn main() {
//     tests::test_parser();
// }

// 在编写时的 错误检查
// language server

fn cheack_file() {
    fn cheack_typedef() {}
}

// 解析 cbml 文件到 编程语言自己的类型 T
fn parse<T>() {}

// language server
// lib

#[allow(dead_code)]
fn timeit(count: usize, f: fn()) {
    use std::time::Instant;
    let start = Instant::now(); // 记录开始时间

    for _ in 0..count {
        f();
    }

    let duration = start.elapsed(); // 计算耗时

    println!("耗时：{:?}", duration);
}

#[cfg(test)]
mod tests {

    use crate::{dp, lexer::tokenizer, typecheck::typecheck};

    #[test]
    pub fn test_parser() {
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/3_enum.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/4_number.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/5_string.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/6_optinal.cbml");

        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/7_struct.cbml");

        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/8_union.cbml");
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
        dsafdasfsadf(path, &code);
    }

    fn dsafdasfsadf(path: &str, code: &str) {
        use crate::parser::cbml_parser::CbmlParser;

        let tokens = tokenizer(path, code)
            .map_err(|e| {
                println!("{:?}", e);
                return e;
            })
            .unwrap();

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(path.to_string(), &tokens);
        let re = parser.parse();

        drop(tokens);

        match re {
            Ok(ref ast) => {
                // ast.iter().for_each(|s| {
                //     dp(format!("statement: {:?}", s));
                // });

                // dp("start typecheck: ");

                let re = typecheck(path.into(), ast);

                if re.is_empty() {
                    dp("没有检查出类型错误.");
                } else {
                    // has errors.
                    re.iter().for_each(|x| {
                        x.report_error(code);
                        // dp(format!("{:?}", x));
                    });
                }
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    dp(format!("message: {:?}", s.msg));
                    // dp(format!("tok: {:?}", s.token));
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
