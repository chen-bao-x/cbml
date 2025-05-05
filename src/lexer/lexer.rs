use super::token::{Location, Position, Token, TokenKind as tk};
use crate::dp;
use std::num::ParseFloatError;

#[derive(Debug, Clone, Copy)]
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

    start_pos: Option<Position>,
    end_pos: Option<Position>,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        Lexer {
            current: String::new(),
            input: code.chars().collect(),
            position: 0,
            line: 1,
            column: 0,
            state: State::Initial,
            start_pos: None,
            end_pos: None,
        }
    }

    fn push(&mut self, ch: char) {
        self.current.push(ch);
        self.advance().unwrap();
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.input.get(self.position) {
            self.position += 1;
            if *ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            return Some(*ch);
        } else {
            return None;
        }
    }

    // fn fall_back(&mut self) {
    //     if let Some(ch) = self.input.get(self.position) {
    //         self.position -= 1;
    //         if *ch == '\n' {
    //             self.line -= 1;
    //             self.column = 1; // todo!() 上一行的最后一个 column
    //         } else {
    //             self.column -= 1;
    //         }
    //         // return Some(*ch);
    //     } else {
    //         // return None;
    //         todo!();
    //     }
    // }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    // fn peek_next(&self, n: usize) -> Option<char> {
    //     self.input.get(self.position + n).copied()
    // }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            dp(format!("{ch}"));
            dp(format!("{:?}", self.get_current_position()));

            match self.state {
                State::Initial => {
                    match ch {
                        ' ' | '\t' => {
                            /* skip whitespace */
                            self.advance(); // eat this character
                        }
                        '\n' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::NewLine, loc));
                        }
                        '(' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::LParen, loc));
                        }
                        ')' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::RParen, loc));
                        }
                        '[' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::LBracket, loc));
                        }
                        ']' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::RBracket, loc));
                        }
                        '{' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::LBrace, loc));
                        }
                        '}' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::RBrace, loc));
                        }
                        ',' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Comma, loc));
                        }
                        ':' => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Colon, loc));
                        }
                        '|' => {
                            self.advance(); // eat this character
                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Pipe, loc));
                        }
                        '=' => {
                            self.advance(); // eat this character
                            self.mark_start_pos();
                            let loc = self.get_pos();
                            tokens.push(Token::new(tk::Asign, loc));
                        }
                        '?' => {
                            self.advance(); // eat this character
                            self.mark_start_pos();
                            let loc = self.get_pos();
                            tokens.push(Token::new(tk::QuestionMark, loc));
                        }

                        '"' => {
                            self.advance(); // eat this character
                            // self.push(ch);
                            self.mark_start_pos();

                            self.state = State::InString;
                            self.current.clear();
                        }
                        '/' => {
                            self.state = State::InLineComment;
                            self.current.clear();

                            self.push(ch);
                        }

                        '0'..='9' => {
                            self.state = State::InNumber;
                            self.current.clear();

                            self.push(ch);
                        }
                        '+' => {
                            // 正数

                            self.push(ch);
                            self.state = State::InNumber;
                        }
                        '-' => {
                            // 负数
                            self.push(ch);
                            self.state = State::InNumber;
                        }

                        // 处理标识符，支持多语言字符
                        x if x.is_alphanumeric() => {
                            self.current.clear();
                            self.push(ch);
                            self.state = State::InIdentifier;
                        }
                        // 处理无效字符
                        x => {
                            self.advance(); // eat this character

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            // tokens.push(Token::new(tk::Invalid(x), start, end));

                            let tok = Token::new(tk::Invalid(x), loc);
                            return Err(format!("未识别的字符 {:?}", tok));
                        }
                    }
                }
                State::InIdentifier => {
                    match ch {
                        x if x.is_alphanumeric() || x == '_' => {
                            self.push(ch);
                        }

                        _ => {
                            // let tok = Token::new(
                            //     tk::Identifier(std::mem::take(&mut self.current)).handle_keyword(),
                            //     self.line,
                            //     self.column,
                            // );

                            // tokens.push(tok);

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            let identifier = std::mem::take(&mut self.current);

                            tokens
                                .push(Token::new(tk::Identifier(identifier).handle_keyword(), loc));

                            self.state = State::Initial;

                            // self.fall_back();

                            self.current.clear();
                        }
                    }
                }
                State::BinarayNumber => {
                    match ch {
                        '0' | '1' => {
                            self.push(ch);
                            self.advance().unwrap();
                        }
                        _ => {
                            let 是负数吗: bool = {
                                if &self.current[0..1] == "-" {
                                    let _ = self.current.remove(0);
                                    true
                                } else {
                                    false
                                }
                            };

                            let binary_value = u64::from_str_radix(&self.current[2..], 2)
                                .map_err(|e| e.to_string())?;

                            let binary_value = if 是负数吗 {
                                (binary_value as f64) * (-1.0)
                            } else {
                                binary_value as f64
                            };

                            // let tok =
                            //     Token::new(tk::Number(binary_value as f64), self.line, self.column);

                            //     tokens.push(tok);

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Number(binary_value as f64), loc));

                            self.state = State::Initial;

                            // self.fall_back();

                            self.current.clear();
                        }
                    };
                }
                State::HexNumber => match ch {
                    '0'..='9' | 'a'..='f' | 'A'..='F' => {
                        self.push(ch);
                    }
                    _ => {
                        dp(format!("hex {:?}", self.current));

                        let 是负数吗: bool = {
                            if &self.current[0..1] == "-" {
                                let _ = self.current.remove(0);
                                true
                            } else {
                                false
                            }
                        };

                        let hex_value: f64 = {
                            let v = u64::from_str_radix(&self.current[2..], 16)
                                .map_err(|e| e.to_string())?;

                            if 是负数吗 {
                                (v as f64) * (-1.0)
                            } else {
                                v as f64
                            }
                        };

                        // let tok = Token::new(tk::Number(hex_value as f64), self.line, self.column);

                        // tokens.push(tok);

                        self.mark_start_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(tk::Number(hex_value as f64), loc));

                        self.state = State::Initial;
                        // self.fall_back();
                        self.current.clear();
                    }
                },
                State::InNumber => {
                    match ch {
                        '0'..='9' => {
                            self.push(ch);
                        }
                        '.' => {
                            match self.current.find(|x| x == '.') {
                                Some(_) => {
                                    // 已经有小数点了, 不能重复出现小数点.
                                    return Err(format!("无效的数字格式 {:?}", self.current));
                                }
                                None => {
                                    // 处理小数点
                                    self.push(ch);
                                }
                            }
                        }
                        'x' => {
                            self.push(ch);
                            self.state = State::HexNumber;
                        }
                        'b' => {
                            self.push(ch);
                            self.state = State::BinarayNumber;
                        }
                        _ => {
                            // 处理数字结束
                            let num: f64 = self
                                .current
                                .parse()
                                .map_err(|e: ParseFloatError| e.to_string())?;

                            // let tok = Token::new(tk::Number(num), self.line, self.column);
                            // tokens.push(tok);

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Number(num), loc));

                            self.state = State::Initial;
                            // self.fall_back();
                            self.current.clear();
                        }
                    };
                }
                // State::InOptator => todo!(),
                State::InString => {
                    match ch {
                        '"' => {
                            // string ends.
                            self.advance();

                            self.mark_end_pos();

                            let loc = self.get_pos();

                            // self.current 存放 String 字面量的内容.

                            tokens.push(Token::new(
                                tk::String(std::mem::take(&mut self.current)),
                                loc,
                            ));

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        '\\' => {
                            self.advance();
                            // 处理转义字符

                            if let Some(next_ch) = self.peek() {
                                match next_ch {
                                    'n' => {
                                        self.push('\n');
                                    }
                                    'r' => {
                                        self.push('\r');
                                    }
                                    't' => {
                                        self.push('\r');
                                    }
                                    'u' => {
                                        // Unicode 转义 "\u{1F600}"

                                        self.advance(); // 

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
                                        // for _ in 0..4 {
                                        for _ in 0..10 {
                                            // \u{1F600} // 大括号中的 hex number 字符数量暂时设置为不超过 10 个.

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

                                            self.push(sadf);
                                        } else {
                                            return Err(format!(
                                                "无效的 unicode 转义序列 {:?}",
                                                unicode
                                            ));
                                        }
                                    }
                                    '\\' => {
                                        self.push('\\');
                                        self.advance();
                                    }
                                    '"' => {
                                        self.push('\"');
                                        self.advance();
                                    }
                                    '0' => {
                                        self.push('\0');
                                        self.advance();
                                    }
                                    _ => {
                                        self.push(ch);
                                    }
                                }
                            }
                        }
                        _ => {
                            self.push(ch);
                        }
                    }
                }
                State::InLineComment => {
                    match ch {
                        '\n' => {
                            // 结束注释

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(
                                tk::LineComment(std::mem::take(&mut self.current)),
                                loc,
                            ));

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        '/' => {
                            // 单行注释

                            self.push(ch);

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

                            self.push(ch);
                            self.state = State::InBlockComment;
                        }
                        c => {
                            self.push(c);
                        }
                    }
                }
                State::InDocComment => {
                    match ch {
                        '\n' => {
                            // 结束注释

                            self.mark_start_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(
                                tk::DocComment(std::mem::take(&mut self.current)),
                                loc,
                            ));

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        x => {
                            self.push(x);
                        }
                    }
                }
                State::InBlockComment => {
                    match ch {
                        '*' => {
                            self.push(ch);
                            if let Some(next_ch) = self.peek() {
                                if next_ch == '/' {
                                    // 结束注释
                                    self.advance();

                                    self.push('/');

                                    self.mark_start_pos();
                                    let loc = self.get_pos();

                                    tokens.push(Token::new(
                                        tk::BlockComment(std::mem::take(&mut self.current)),
                                        loc,
                                    ));

                                    self.state = State::Initial;
                                    self.current.clear();
                                }
                            } else {
                                return Err(format!("语法错误: 需要一个 /",));
                            }
                        }
                        x => {
                            self.push(x);
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
                        self.mark_start_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::Identifier(std::mem::take(&mut self.current)).handle_keyword(),
                            loc,
                        ));

                        self.current.clear();
                    }
                    State::InNumber => {
                        self.mark_start_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::Number(
                                self.current
                                    .parse()
                                    .map_err(|e: ParseFloatError| e.to_string())?,
                            ),
                            loc,
                        ));

                        self.current.clear();
                    }

                    State::HexNumber => {
                        let 是负数吗: bool = {
                            if &self.current[0..1] == "-" {
                                let _ = self.current.remove(0);
                                true
                            } else {
                                false
                            }
                        };

                        let hex_value: f64 = {
                            let v = u64::from_str_radix(&self.current[2..], 16)
                                .map_err(|e| e.to_string())?;

                            if 是负数吗 {
                                (v as f64) * (-1.0)
                            } else {
                                v as f64
                            }
                        };

                        self.mark_start_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(tk::Number(hex_value as f64), loc));

                        self.current.clear();
                    }

                    State::BinarayNumber => {
                        let 是负数吗: bool = {
                            if &self.current[0..1] == "-" {
                                let _ = self.current.remove(0);
                                true
                            } else {
                                false
                            }
                        };

                        let binary_value = u64::from_str_radix(&self.current[2..], 2)
                            .map_err(|e| e.to_string())?;

                        let binary_value = if 是负数吗 {
                            (binary_value as f64) * (-1.0)
                        } else {
                            binary_value as f64
                        };

                        self.mark_start_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(tk::Number(binary_value as f64), loc));

                        self.current.clear();
                    }

                    State::InLineComment => {
                        self.mark_start_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::LineComment(std::mem::take(&mut self.current)),
                            loc,
                        ));
                    }
                    // State::Initial => todo!(),
                    // State::InString => todo!(),
                    // State::InDocComment => todo!(),
                    // State::InBlockComment => todo!(),
                    _ => {
                        dp(format!("{:?}", self.state));
                        dp(format!("{:?}", self.current));
                        todo!();
                    }
                }
            }
        }

        return Ok(tokens);
    }
}

impl Lexer {
    fn get_current_position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
        }
    }

    fn get_pos(&mut self) -> Location {
        let start = match self.start_pos.clone() {
            Some(p) => p,
            None => todo!(),
        };
        let end = match self.end_pos.clone() {
            Some(p) => p,
            None => self.get_current_position(),
        };

        self.start_pos = None;
        self.end_pos = None;

        return Location { start, end };
    }

    fn mark_start_pos(&mut self) {
        self.start_pos = self.get_current_position().into();
    }

    fn mark_end_pos(&mut self) {
        self.end_pos = self.get_current_position().into();
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
                    dp(format!("{:?}", token.kind));
                }
            }
            Err(e) => {
                dp(format!("Error: {}", e));
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
