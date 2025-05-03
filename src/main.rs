mod lexer;
mod parser;
fn main() {
    lexer::tokenizer("name = \"hello\"").unwrap();
}

#[test]
fn test_parser() {
    // let code = std::fs::read_to_string("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml").unwrap();
    // let code = std::fs::read_to_string("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml") .unwrap();

    let code = CODE;
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

const CODE: &str = r##"


package: {
name: string default "hello"
version: string 
edition: string 
}

dependencie: [dependencie]

struct dependencie_with_ssh {
name: string 
git: string 
branch: string 
}

struct dependencie_whith_version {
name:string 
varsion: string 
}

enum dependencie {
ssh({
    name: string 
    git: string 
    branch: string 
}),
version(dependencie_whith_version),
}

"##;
