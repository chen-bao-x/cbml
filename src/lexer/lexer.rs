use std::num::ParseFloatError;

use super::token::{Token, TokenKind};

#[derive(Debug)]
enum State {
    Initial,
    InIdentifier,
    BinarayNumber,
    HexNumber,
    InNumber, // 十进制数字.
    InString,
    InDocComment,   // 文档注释 ///
    InBlockComment, // 块注释 /* */
    InLineComment,  // 单行注释 //
}
#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    state: State,
    current: String,

    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        Lexer {
            current: String::new(),
            input: code.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            state: State::Initial,
        }
    }
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.input.get(self.position) {
            self.position += 1;
            if *ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            return Some(*ch);
        } else {
            return None;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.advance() {
            match self.state {
                State::Initial => {
                    match ch {
                        ' ' | '\t' => { /* skip whitespace */ }
                        '\n' => tokens.push(Token::new(TokenKind::NewLine, self.line, self.column)),
                        '(' => tokens.push(Token::new(TokenKind::LParen, self.line, self.column)),
                        ')' => tokens.push(Token::new(TokenKind::RParen, self.line, self.column)),
                        '[' => tokens.push(Token::new(TokenKind::LBracket, self.line, self.column)),
                        ']' => tokens.push(Token::new(TokenKind::RBracket, self.line, self.column)),
                        '{' => tokens.push(Token::new(TokenKind::LBrace, self.line, self.column)),
                        '}' => tokens.push(Token::new(TokenKind::RBrace, self.line, self.column)),
                        ',' => tokens.push(Token::new(TokenKind::Comma, self.line, self.column)),
                        ':' => tokens.push(Token::new(TokenKind::Colon, self.line, self.column)),
                        '|' => tokens.push(Token::new(TokenKind::Pipe, self.line, self.column)),
                        '=' => tokens.push(Token::new(TokenKind::Asign, self.line, self.column)),
                        '?' => tokens.push(Token::new(TokenKind::QuestionMark, self.line, self.column)),

                        '"' => {
                            self.state = State::InString;
                            self.current.clear();
                            // self.current.push(ch);
                        }
                        '/' => {
                            self.state = State::InLineComment;
                            self.current.clear();
                            self.current.push(ch);
                        }

                        '0'..='9' => {
                            self.state = State::InNumber;
                            self.current.clear();
                            self.current.push(ch);
                        }
                        '+' => {
                            // 正数
                            self.current.push(ch);
                            self.state = State::InNumber;
                        }
                        '-' => {
                            // 负数
                            self.current.push(ch);
                            self.state = State::InNumber;
                        }

                        // 处理标识符，支持多语言字符
                        x if x.is_alphanumeric() => {
                            self.current.clear();
                            self.current.push(ch);
                            self.state = State::InIdentifier;
                        }
                        // 处理无效字符
                        x => {
                            let tok = Token {
                                kind: TokenKind::Invalid(x),
                                line: self.line,
                                column: self.column,
                            };

                            tokens.push(tok.clone());

                            return Err(format!("未识别的字符 {:?}", tok));
                        }
                    }
                }
                State::InIdentifier => {
                    match ch {
                        x if x.is_alphanumeric() || x == '_' => {
                            self.current.push(ch); // 使用当前字符而不是调用 advance()
                        }

                        _ => {
                            let tok = Token::new(
                                TokenKind::Identifier(self.current.clone()).handle_keyword(),
                                self.line,
                                self.column,
                            );

                            tokens.push(tok);

                            self.state = State::Initial;
                            self.position -= 1;
                            self.column -= 1;
                            self.current.clear();
                        }
                    }
                }
                State::BinarayNumber => {
                    match ch {
                        '0' | '1' => {
                            self.current.push(ch);
                        }
                        _ => {
                            let binary_value = u64::from_str_radix(&self.current[2..], 2)
                                .map_err(|e| e.to_string())?;
                            let tok = Token::new(
                                TokenKind::Number(binary_value as f64),
                                self.line,
                                self.column,
                            );

                            tokens.push(tok);

                            self.state = State::Initial;
                            self.position -= 1;
                            self.column -= 1;
                            self.current.clear();
                        }
                    };
                }
                State::HexNumber => match ch {
                    '0'..='9' | 'a'..='f' | 'A'..='F' => {
                        self.current.push(ch);
                    }
                    _ => {
                        println!("hex {:?}", self.current);
                        let hex_value = u64::from_str_radix(&self.current[2..], 16)
                            .map_err(|e| e.to_string())?;
                        let tok =
                            Token::new(TokenKind::Number(hex_value as f64), self.line, self.column);

                        tokens.push(tok);

                        self.state = State::Initial;
                        self.position -= 1;
                        self.column -= 1;
                        self.current.clear();
                    }
                },
                State::InNumber => {
                    match ch {
                        '0'..='9' => {
                            self.current.push(ch);
                        }
                        '.' => {
                            match self.current.find(|x| x == '.') {
                                Some(_) => {
                                    // 已经有小数点了, 不能重复出现小数点.
                                    return Err(format!("无效的数字格式 {:?}", self.current));
                                }
                                None => {
                                    // 处理小数点
                                    self.current.push(ch);
                                }
                            }
                        }
                        'x' => {
                            self.current.push(ch);
                            self.state = State::HexNumber;
                        }
                        'b' => {
                            self.current.push(ch);
                            self.state = State::BinarayNumber;
                        }
                        _ => {
                            // 处理数字结束
                            let num: f64 = self
                                .current
                                .parse()
                                .map_err(|e: ParseFloatError| e.to_string())?;

                            let tok = Token::new(TokenKind::Number(num), self.line, self.column);
                            tokens.push(tok);

                            self.state = State::Initial;
                            self.position -= 1;
                            self.column -= 1;
                            self.current.clear();
                        }
                    };
                }
                // State::InOptator => todo!(),
                State::InString => {
                    match ch {
                        '"' => {
                            // self.current.push(ch);
                            let tok = Token::new(
                                TokenKind::String(self.current.clone()),
                                self.line,
                                self.column,
                            );
                            tokens.push(tok);
                            self.state = State::Initial;
                            self.current.clear();
                        }
                        '\\' => {
                            // 处理转义字符
                            if let Some(next_ch) = self.peek() {
                                match next_ch {
                                    'n' => {
                                        self.current.push('\n');
                                        self.advance();
                                    }
                                    't' => {
                                        self.current.push('\t');
                                        self.advance();
                                    }
                                    'u' => {
                                        // Unicode 转义
                                        self.current.push(ch);
                                        self.advance();

                                        let mut unicode = String::new();
                                        if let Some(next_ch) = self.peek() {
                                            if next_ch == '{' {
                                                self.advance();
                                            } else {
                                                return Err(format!(
                                                    "需要一个 {{ , 而不是 {:?}",
                                                    next_ch
                                                ));
                                            }
                                        } else {
                                            return Err(format!(
                                                "需要一个 {{ , 而不是 {:?}",
                                                next_ch
                                            ));
                                        }
                                        for _ in 0..4 {
                                            if let Some(next_ch) = self.peek() {
                                                if next_ch == '}' {
                                                    self.advance();
                                                    break;
                                                } else {
                                                    unicode.push(next_ch);
                                                    self.advance();
                                                }
                                            } else {
                                                return Err(format!(
                                                    "需要一个 }} , 而不是 {:?}",
                                                    next_ch
                                                ));
                                            }
                                        }

                                        if let Ok(unicode) = u32::from_str_radix(&unicode, 16) {
                                            let sadf = match std::char::from_u32(unicode) {
                                                Some(ch) => ch,
                                                None => {
                                                    return Err(format!(
                                                        "无效的 unicode 转义序列 {:?}",
                                                        unicode
                                                    ));
                                                }
                                            };

                                            self.current.push(sadf);
                                        } else {
                                            return Err(format!(
                                                "无效的 unicode 转义序列 {:?}",
                                                unicode
                                            ));
                                        }
                                    }
                                    _ => {
                                        self.current.push(ch);
                                    }
                                }
                            }
                        }
                        _ => {
                            self.current.push(ch);
                        }
                    }
                }
                State::InLineComment => {
                    match ch {
                        '\n' => {
                            // 结束注释

                            let tok = Token::new(
                                TokenKind::LineComment(self.current.clone()),
                                self.line,
                                self.column,
                            );
                            tokens.push(tok);

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        '/' => {
                            // 单行注释
                            self.current.push(ch);

                            match self.current.as_str() {
                                "//" => {
                                    self.state = State::InLineComment;
                                }
                                "///" => {
                                    self.state = State::InDocComment;
                                }
                                _ => {}
                            }
                        }
                        '*' => {
                            // 多行注释
                            self.current.push(ch);
                            self.state = State::InBlockComment;
                        }
                        c => {
                            self.current.push(c);
                        }
                    }
                }
                State::InDocComment => {
                    match ch {
                        '\n' => {
                            // 结束注释
                            let tok = Token::new(
                                TokenKind::DocComment(self.current.clone()),
                                self.line,
                                self.column,
                            );
                            tokens.push(tok);

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        x => {
                            self.current.push(x);
                        }
                    }
                }
                State::InBlockComment => {
                    match ch {
                        '*' => {
                            self.current.push(ch);
                            if let Some(next_ch) = self.peek() {
                                if next_ch == '/' {
                                    // 结束注释
                                    self.advance();
                                    self.current.push('/');

                                    let tok = Token::new(
                                        TokenKind::BlockComment(self.current.clone()),
                                        self.line,
                                        self.column,
                                    );
                                    tokens.push(tok);

                                    self.state = State::Initial;
                                    self.current.clear();
                                }
                            } else {
                                return Err(format!("语法错误: 需要一个 /",));
                            }
                        }
                        x => {
                            self.current.push(x);
                        }
                    }
                }
            }
        }

        {
            // 处理最后的状态
            if !self.current.is_empty() {
                match self.state {
                    State::InIdentifier => {
                        let tok = Token {
                            kind: TokenKind::Identifier(self.current.clone()),
                            line: self.line,
                            column: self.column,
                        };
                        tokens.push(tok);
                        self.current.clear();
                    }
                    State::InNumber => {
                        let tok = Token {
                            // kind: TokenKind::HexNumber(self.current.clone()),
                            kind: TokenKind::Number(
                                self.current
                                    .parse()
                                    .map_err(|e: ParseFloatError| e.to_string())?,
                            ),
                            line: self.line,
                            column: self.column,
                        };
                        tokens.push(tok);
                        self.current.clear();
                    }
                    // State::InString => {
                    //     tokens.push(Token::new(TokenKind::Invalid('"'), &self));
                    // }
                    // State::InChar => {
                    //     tokens.push(Token::new(TokenKind::Invalid('\''), &self));
                    // }
                    _ => {
                        println!("{:?}", self.state);
                        println!("{:?}", self.current);
                        todo!();
                    }
                }
            }
        }

        return Ok(tokens);
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn sadf() {
        sadfdasf(EXMAPLE_1);
        // sadfdasf(exmaple_2);
    }

    fn sadfdasf(code: &str) {
        let a = Lexer::new(code).tokenize();
        match a {
            Ok(tokens) => {
                for token in tokens {
                    println!("{:?}", token.kind);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    const EXMAPLE_1: &str = r###"
    use "./1.typedef.cbml"
// line comment 
/// doc comment 
/* 
muilty line comment 
*/
package = {
    name = "new"
    version = "0.1.0"
    edition = "2021"
}

dependencies = [
    { name = "chenbao_cmd", git = "ssh://git@github.com/chen-bao-x/chenbao_cmd.git", branch = "master" },
    { name = "colored", varsion = "3.0.0", }
]
 
 0b10101010101010101010101010101010
 0x1010
 0xffaa1f
 0b10101010
 0b10101010


str: string 
str = "string"
num: number 
num = -1324

    "###;

    const EXMAPLE_2: &str = r###"


    package: {
        name: string 
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
    
    "###;
}
