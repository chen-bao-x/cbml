pub mod cbml_project;
pub mod cbml_data;
pub mod formater;
pub mod lexer;
pub mod parser;

// fn main() {
//     tests::test_parser();
// }

// 在编写时的 错误检查
// language server

// fn cheack_file() {
//     fn cheack_typedef() {}
// }

// 解析 cbml 文件到 编程语言自己的类型 T
// fn parse<T>() {}

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

    use crate::cbml_project::{code_file::CodeFile, typedef_file::TypedefFile};

    #[test]
    pub fn test_parser() {
        // /Users/chenbao/GitHub/cbml/examples/1.cmml

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/1.cbml");
        test_code_file("/Users/chenbao/GitHub/cbml/examples/1.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/1.def.cbml");
        test_typedef("/Users/chenbao/GitHub/cbml/examples/1.def.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/2_arr.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/3_enum.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/4_number.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/5_string.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/6_optinal.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/7_struct.cbml");

        // asdfasdfsdf("/Users/chenbao/GitHub/cbml/examples/8_union.cbml");
    }

    fn test_code_file(path: &str) {
        use std::fs::read_to_string;
        let code = read_to_string(path).unwrap();

        let asdf = CodeFile::new(path.into());
        asdf.errors.iter().for_each(|x| {
            x.report_error(&code);
        });

        println!("asdf.typedef_file");
        asdf.typedef_file
            .unwrap()
            .fields_map
            .iter()
            .for_each(|ref x| println!("name: {}, scope: {}", x.1.name, x.1.scope.0));

        println!("asdf.fields");
        asdf.fields
            .iter()
            .for_each(|ref x| println!("name: {}, scope: {}", x.name, x.scope.0));
    }

    fn test_typedef(path: &str) {
        use std::fs::read_to_string;
        let code = read_to_string(path).unwrap();

        let asdf = TypedefFile::new(path.into());
        asdf.errors.iter().for_each(|x| {
            x.report_error(&code);
        });
    }

    // fn asdfasdfsdf(path: &str) {
    //     use std::fs::read_to_string;
    //     let code = read_to_string(path).unwrap();
    //     dsafdasfsadf(path, &code);
    // }

    // fn dsafdasfsadf(path: &str, code: &str) {
    //     use crate::parser::cbml_parser::CbmlParser;

    //     let lexer_result = tokenizer(path, code);

    //     lexer_result.errors.iter().for_each(|x| {
    //         println!("{}", x.lookup(code));
    //     });

    //     let tokens = lexer_result.tokens;

    //     // dp(format!("tokens: {:?}", tokens));

    //     let mut parser = CbmlParser::new(path.to_string(), &tokens);
    //     let re = parser.parse();

    //     drop(tokens);

    //     if !re.errors.is_empty() {
    //         re.errors.iter().for_each(|s| {
    //             dp(format!("message: {:?}", s.msg));
    //             // dp(format!("tok: {:?}", s.token));
    //         });
    //     }
    // }
}

/// debug_println.
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
