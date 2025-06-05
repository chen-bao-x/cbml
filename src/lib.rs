mod cbml_codable;

pub use cbml_codable::*;
pub mod cbml_data;
pub use cbml_data::cbml_type::*;
pub use cbml_data::cbml_value::*;
pub use cbml_root::*;
pub mod cbml_project;
pub mod lexer;
pub mod parser;

/// 输出为 cbml 源代码.
pub trait ToCbml {
    /// 将数据转换为 cbml 源代码.
    /// deepth -> 缩进深度.
    fn to_cbml(&self, deepth: usize) -> String;
}

pub fn from_cbml<T>(cbml_code: &str) -> Result<T, ()>
where
    T: CbmlCodable,
{
    let f = crate::cbml_project::code_file::CodeFile::new_from("".to_string(), cbml_code);
    let val = f.to_cbml_value();
    let result = T::from_cbml_value(val);
    return result;
}

pub fn to_cbml<T>(value: T) -> String
where
    T: CbmlCodable,
{
    value.to_cbml_value().to_cbml(0)
}

#[allow(dead_code)]
pub trait ToCbmlType {
    fn to_cbml_type(&self) -> CbmlType;
}

/// convert to CbmlValue.
pub trait ToCbmlValue {
    fn to_cbml_value(&self) -> CbmlValue;
}

// fn main() {
//     tests::test_parser();
// }

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
            .for_each(|ref x| println!("name: {}, scope: {}", x.1.name, x.1.scope_id.0));

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

use std::collections::HashMap;
pub trait AndThenTo<'a> {
    fn cbml_str(&self) -> Option<&'a str>;
    fn cbml_number(&self) -> Option<f64>;
    fn cbml_bool(&self) -> Option<bool>;
    fn cbml_none(&self) -> Option<CbmlNoneValue>;
    fn cbml_array(&self) -> Option<&'a Vec<CbmlValue>>;
    fn cbml_struct(&self) -> Option<&'a HashMap<String, CbmlValue>>;
    fn cbml_enum_field(&self) -> Option<(String, Box<CbmlValue>)>;
    fn look_up(&self) -> &Self;
}

// .def.cbml -> rust code
// typedef_file.to_rust_type()

// rust code -> .def.cbml
// typedef_file.from_rust_type()

//
//

// .cbml -> rust data
// code_file.to::<T>()

// rust data -> .cbml
// code_file.from::<T>()

struct MyConfigsadfasdfsdaf {
    string: String,
    number: f64,
    boolean: bool,
    obj: Obj,
    opt_string: Option<String>,
    opt_number: Option<f64>,
    opt_boolean: Option<bool>,
    opt_obj: Option<Obj>,
    vec_string: Vec<String>,
    vec_number: Vec<f64>,
    vec_boolean: Vec<bool>,
    vec_obj: Vec<Obj>,
    // enu:
}

impl MyConfigsadfasdfsdaf {
    fn to_cbml() -> String {
        todo!()
    }
    
    fn from_cbml_file(file_path: &str) {
        let f = crate::cbml_project::code_file::CodeFile::new(file_path.into());

    }

    fn from_cbml(code: &str) {}
}

struct Obj {
    haha_name: String,
}
