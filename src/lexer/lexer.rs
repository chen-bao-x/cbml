use crate::parser::parser_error::ParserError;

use super::token::{Position, Span, Token, TokenID};

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
    file_path: String,
    input: Vec<char>,
    state: State,
    current: String,

    position: usize,
    line: usize,
    column: usize,

    start_pos: Option<Position>,
    end_pos: Option<Position>,

    token_id: TokenID,
}

pub struct LexerResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<ParserError>,
}

impl Lexer {
    pub fn new(file_path: &str, code: &str) -> Self {
        Lexer {
            file_path: file_path.into(),
            current: String::new(),
            input: code.chars().collect(),
            position: 0,
            line: 0,
            column: 0,
            state: State::Initial,
            start_pos: None,
            end_pos: None,
            token_id: TokenID(0),
        }
    }

    fn gen_token_id(&mut self) -> TokenID {
        self.token_id.0 += 1;
        return self.token_id;
    }

    fn push_and_advance(&mut self, ch: char) {
        self.current.push(ch);

        let re = self.advance();
        match re {
            Some(_) => {}
            None => {
                panic!("adsfasdfsadfsafd");
            }
        };
    }

    fn advance(&mut self) -> Option<char> {
        // println!(
        //     "self.advance pos: {}, line: {}, colunm:{}, state: {:?}",
        //     self.position, self.line, self.column, self.state
        // );

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

    // pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
    // pub fn tokenize(&mut self) -> Result<Vec<Token>, ParserError> {
    pub fn tokenize(&mut self) -> LexerResult {
        use crate::lexer::token::TokenKind as tk;

        let mut tokens = Vec::new();

        let mut errors: Vec<ParserError> = Vec::new();

        while let Some(ch) = self.peek() {
            match self.state {
                State::Initial => {
                    match ch {
                        ' ' | '\t' => {
                            /* skip whitespace */
                            self.advance(); // eat this character
                        }
                        '\n' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();

                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::NewLine, loc, self.gen_token_id()));
                        }
                        '(' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::LParen, loc, self.gen_token_id()));
                        }
                        ')' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::RParen, loc, self.gen_token_id()));
                        }
                        '[' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::LBracket, loc, self.gen_token_id()));
                        }
                        ']' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::RBracket, loc, self.gen_token_id()));
                        }
                        '{' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::LBrace, loc, self.gen_token_id()));
                        }
                        '}' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::RBrace, loc, self.gen_token_id()));
                        }
                        ',' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Comma, loc, self.gen_token_id()));
                        }
                        ':' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Colon, loc, self.gen_token_id()));
                        }
                        '|' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Pipe, loc, self.gen_token_id()));
                        }
                        '=' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();
                            tokens.push(Token::new(tk::Asign, loc, self.gen_token_id()));
                        }
                        '?' => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();
                            let loc = self.get_pos();
                            tokens.push(Token::new(tk::QuestionMark, loc, self.gen_token_id()));
                        }

                        '"' => {
                            self.current.clear();

                            self.mark_start_pos();
                            self.push_and_advance(ch);

                            self.state = State::InString;
                        }
                        '/' => {
                            self.current.clear();
                            self.mark_start_pos();

                            self.push_and_advance(ch);

                            self.state = State::InLineComment;
                        }

                        '0'..='9' => {
                            self.state = State::InNumber;
                            self.current.clear();

                            self.mark_start_pos();
                            self.push_and_advance(ch);
                        }
                        '+' => {
                            // 正数
                            self.mark_start_pos();
                            self.push_and_advance(ch);
                            self.state = State::InNumber;
                        }
                        '-' => {
                            // 负数
                            self.mark_start_pos();
                            self.push_and_advance(ch);
                            self.state = State::InNumber;
                        }

                        // 处理标识符，支持多语言字符
                        x if x.is_alphanumeric() => {
                            self.current.clear();

                            self.mark_start_pos();
                            self.push_and_advance(ch);
                            self.state = State::InIdentifier;
                        }
                        // 处理无效字符
                        x => {
                            self.mark_start_pos();
                            self.advance(); // eat this character
                            self.mark_end_pos();

                            let loc = self.get_pos();

                            // tokens.push(Token::new(tk::Invalid(x), start, end));

                            let tok = Token::new(tk::Invalid(x), loc, self.gen_token_id());

                            errors.push(ParserError {
                                file_path: self.file_path.clone(),
                                msg: format!("未识别的字符 {}", x),
                                code_location: tok.span,
                                note: None,
                                help: None,
                            });
                        }
                    }
                }
                State::InIdentifier => match ch {
                    x if x.is_alphanumeric() || x == '_' => {
                        self.push_and_advance(ch);
                    }

                    _ => {
                        self.mark_end_pos();

                        let identifier = std::mem::take(&mut self.current);
                        tokens.push(Token::new(
                            tk::Identifier(identifier).handle_keyword(),
                            self.get_pos(),
                            self.gen_token_id(),
                        ));

                        self.state = State::Initial;
                        self.current.clear();
                    }
                },
                State::BinarayNumber => {
                    match ch {
                        '0' | '1' => {
                            self.push_and_advance(ch);
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

                            // let binary_value =
                            //     u64::from_str_radix(&self.current[2..], 2).map_err(|e| {
                            //         ParserError {
                            //             file_path: self.file_path.clone(),
                            //             msg: e.to_string(),
                            //             code_location: self.get_pos(),
                            //             note: None,
                            //             help: None,
                            //         }
                            //     })?;
                            let binary_value = match u64::from_str_radix(&self.current[2..], 2) {
                                Ok(f) => f,
                                Err(e) => {
                                    errors.push(ParserError {
                                        file_path: self.file_path.clone(),
                                        msg: e.to_string(),
                                        code_location: self.get_pos(),
                                        note: None,
                                        help: None,
                                    });
                                    continue;
                                }
                            };

                            let binary_value = if 是负数吗 {
                                (binary_value as f64) * (-1.0)
                            } else {
                                binary_value as f64
                            };

                            // let tok =
                            //     Token::new(tk::Number(binary_value as f64), self.line, self.column);

                            //     tokens.push(tok);

                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(
                                tk::Number(binary_value as f64),
                                loc,
                                self.gen_token_id(),
                            ));

                            self.state = State::Initial;

                            // self.fall_back();

                            self.current.clear();
                        }
                    };
                }
                State::HexNumber => match ch {
                    '0'..='9' | 'a'..='f' | 'A'..='F' => {
                        self.push_and_advance(ch);
                    }
                    _ => {
                        // dp(format!("hex {:?}", self.current));

                        let 是负数吗: bool = {
                            if &self.current[0..1] == "-" {
                                let _ = self.current.remove(0);
                                true
                            } else {
                                false
                            }
                        };

                        let hex_value: f64 = {
                            // let v = u64::from_str_radix(&self.current[2..], 16).map_err(|e| {
                            //     ParserError {
                            //         file_path: self.file_path.clone(),
                            //         msg: e.to_string(),
                            //         code_location: self.get_pos(),
                            //         note: None,
                            //         help: None,
                            //     }
                            // })?;
                            let v = match u64::from_str_radix(&self.current[2..], 16) {
                                Ok(f) => f,
                                Err(e) => {
                                    errors.push(ParserError {
                                        file_path: self.file_path.clone(),
                                        msg: e.to_string(),
                                        code_location: self.get_pos(),
                                        note: None,
                                        help: None,
                                    });
                                    continue;
                                }
                            };

                            if 是负数吗 {
                                (v as f64) * (-1.0)
                            } else {
                                v as f64
                            }
                        };

                        // let tok = Token::new(tk::Number(hex_value as f64), self.line, self.column);

                        // tokens.push(tok);

                        self.mark_end_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::Number(hex_value as f64),
                            loc,
                            self.gen_token_id(),
                        ));

                        self.state = State::Initial;
                        // self.fall_back();
                        self.current.clear();
                    }
                },
                State::InNumber => {
                    match ch {
                        '0'..='9' => {
                            self.push_and_advance(ch);
                        }
                        '.' => {
                            match self.current.find(|x| x == '.') {
                                Some(_) => {
                                    // 已经有小数点了, 不能重复出现小数点.
                                    // return Err(ParserError {
                                    //     file_path: self.file_path.clone(),
                                    //     msg: format!("无效的数字格式 {:?}", self.current),
                                    //     code_location: self.get_pos(),
                                    //     note: Some("number 中最多有一个小数点.".into()),
                                    //     help: None,
                                    // });
                                    errors.push(ParserError {
                                        file_path: self.file_path.clone(),
                                        msg: format!("无效的数字格式 {:?}", self.current),
                                        code_location: self.get_pos(),
                                        note: Some("number 中最多有一个小数点.".into()),
                                        help: None,
                                    });
                                }
                                None => {
                                    // 处理小数点
                                    self.push_and_advance(ch);
                                }
                            }
                        }
                        'x' => {
                            self.push_and_advance(ch);
                            self.state = State::HexNumber;
                        }
                        'b' => {
                            self.push_and_advance(ch);
                            self.state = State::BinarayNumber;
                        }
                        _ => {
                            // 处理数字结束
                            // let num: f64 =
                            //     self.current
                            //         .parse()
                            //         .map_err(|e: ParseFloatError| ParserError {
                            //             file_path: self.file_path.clone(),
                            //             msg: e.to_string(),
                            //             code_location: self.get_pos(),
                            //             note: None,
                            //             help: None,
                            //         })?;
                            let num: f64 = match self.current.parse() {
                                Ok(f) => f,
                                Err(e) => {
                                    errors.push(ParserError {
                                        file_path: self.file_path.clone(),
                                        msg: e.to_string(),
                                        code_location: self.get_pos(),
                                        note: None,
                                        help: None,
                                    });
                                    continue;
                                }
                            };

                            // let tok = Token::new(tk::Number(num), self.line, self.column);
                            // tokens.push(tok);

                            self.mark_end_pos();
                            let loc = self.get_pos();

                            tokens.push(Token::new(tk::Number(num), loc, self.gen_token_id()));

                            self.state = State::Initial;

                            self.current.clear();
                        }
                    };
                }
                // State::InOptator => todo!(),
                State::InString => {
                    match ch {
                        '"' => {
                            // string ends.

                            self.push_and_advance(ch);
                            self.mark_end_pos();

                            let loc = self.get_pos();

                            // self.current 存放 String 字面量的内容. 包含 双引号.

                            tokens.push(Token::new(
                                tk::String(std::mem::take(&mut self.current)),
                                loc,
                                self.gen_token_id(),
                            ));

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        '\\' => {
                            // self.advance();
                            // 处理转义字符
                            self.push_and_advance(ch);
                            self.push_and_advance(ch);

                            // 字符的转义交由后续来处理, 这里仅仅将 源代码 中的字符串字面量完整记录下来.

                            // if let Some(next_ch) = self.peek() {
                            //     match next_ch {
                            //         'n' => {
                            //             self.push_and_advance('\n');
                            //         }
                            //         'r' => {
                            //             self.push_and_advance('\r');
                            //         }
                            //         't' => {
                            //             self.push_and_advance('\t');
                            //         }
                            //         'u' => {
                            //             // Unicode 转义 "\u{1F600}"

                            //             self.advance(); //

                            //             let mut unicode = String::new();
                            //             if let Some(next_ch) = self.peek() {
                            //                 if next_ch == '{' {
                            //                     self.advance();
                            //                 } else {
                            //                     return Err(ParserError {
                            //                         file_path: self.file_path.clone(),
                            //                         msg: format!(
                            //                             "需要一个 {{ , 而不是 {:?}",
                            //                             next_ch
                            //                         ),
                            //                         code_location: self.get_pos(),
                            //                         note: None,
                            //                         help: None,
                            //                     });
                            //                 }
                            //             } else {
                            //                 // return Err(format!(
                            //                 //     "需要一个 {{ , 而不是 {:?}",
                            //                 //     next_ch
                            //                 // ));
                            //                 return Err(ParserError {
                            //                     file_path: self.file_path.clone(),
                            //                     msg: format!("需要一个 {{ , 而不是 {:?}", next_ch),
                            //                     code_location: self.get_pos(),
                            //                     note: None,
                            //                     help: None,
                            //                 });
                            //             }
                            //             // for _ in 0..4 {
                            //             for _ in 0..10 {
                            //                 // \u{1F600} // 大括号中的 hex number 字符数量暂时设置为不超过 10 个.

                            //                 if let Some(next_ch) = self.peek() {
                            //                     if next_ch == '}' {
                            //                         self.advance();
                            //                         break;
                            //                     } else {
                            //                         unicode.push(next_ch);
                            //                         self.advance();
                            //                     }
                            //                 } else {
                            //                     return Err(ParserError {
                            //                         file_path: self.file_path.clone(),
                            //                         msg: format!(
                            //                             "需要一个 {{ , 而不是 {:?}",
                            //                             next_ch
                            //                         ),
                            //                         code_location: self.get_pos(),
                            //                         note: None,
                            //                         help: None,
                            //                     });
                            //                 }
                            //             }

                            //             if let Ok(unicode) = u32::from_str_radix(&unicode, 16) {
                            //                 let sadf = match std::char::from_u32(unicode) {
                            //                     Some(ch) => ch,
                            //                     None => {
                            //                         // return Err(format!(
                            //                         //     "无效的 unicode 转义序列 {:?}",
                            //                         //     unicode
                            //                         // ));

                            //                         return Err(ParserError {
                            //                             file_path: self.file_path.clone(),
                            //                             msg: format!(
                            //                                 "无效的 unicode 转义序列 {:?}",
                            //                                 unicode
                            //                             ),
                            //                             code_location: self.get_pos(),
                            //                             note: None,
                            //                             help: None,
                            //                         });
                            //                     }
                            //                 };

                            //                 self.push_and_advance(sadf);
                            //             } else {
                            //                 return Err(ParserError {
                            //                     file_path: self.file_path.clone(),
                            //                     msg: format!(
                            //                         "无效的 unicode 转义序列 {:?}",
                            //                         unicode
                            //                     ),
                            //                     code_location: self.get_pos(),
                            //                     note: None,
                            //                     help: None,
                            //                 });
                            //             }
                            //         }
                            //         '\\' => {
                            //             self.push_and_advance('\\');
                            //             // self.advance();
                            //         }
                            //         '"' => {
                            //             self.push_and_advance('\"');
                            //             // self.advance();
                            //         }
                            //         '0' => {
                            //             self.push_and_advance('\0');
                            //             // self.advance();
                            //         }
                            //         _ => {
                            //             self.push_and_advance(ch);
                            //         }
                            //     }
                            // }
                        }
                        x => {
                            self.push_and_advance(x);
                        }
                    }
                }
                State::InLineComment => {
                    match ch {
                        '\n' => {
                            // 结束注释

                            self.push_and_advance(ch);
                            self.mark_end_pos();

                            let loc = self.get_pos();

                            tokens.push(Token::new(
                                tk::LineComment(std::mem::take(&mut self.current)),
                                loc,
                                self.gen_token_id(),
                            ));

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        '/' => {
                            // 单行注释

                            self.push_and_advance(ch);

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

                            self.push_and_advance(ch);
                            self.state = State::InBlockComment;
                        }
                        c => {
                            self.push_and_advance(c);
                        }
                    }
                }
                State::InDocComment => {
                    // println!("InDocComment {:?}  self.current: {:?}", ch, self.current);

                    match ch {
                        '\n' => {
                            // 结束注释

                            self.push_and_advance(ch);
                            self.mark_end_pos();

                            let loc = self.get_pos();

                            tokens.push(Token::new(
                                tk::DocComment(self.current.clone()),
                                loc,
                                self.gen_token_id(),
                            ));

                            self.state = State::Initial;
                            self.current.clear();
                        }
                        x => {
                            self.push_and_advance(x);
                        }
                    }
                }
                State::InBlockComment => {
                    match ch {
                        '*' => {
                            self.push_and_advance(ch);
                            if let Some(next_ch) = self.peek() {
                                if next_ch == '/' {
                                    // 结束注释
                                    // self.advance();

                                    self.push_and_advance('/');
                                    // let sadf = self.peek();

                                    // println!("{:?}", sadf);
                                    // todo!();

                                    self.mark_end_pos();
                                    let loc = self.get_pos();

                                    tokens.push(Token::new(
                                        tk::BlockComment(std::mem::take(&mut self.current)),
                                        loc,
                                        self.gen_token_id(),
                                    ));

                                    self.state = State::Initial;
                                    self.current.clear();
                                }
                            } else {
                                errors.push(ParserError {
                                    file_path: self.file_path.clone(),
                                    msg: format!("语法错误: 需要一个 /"),
                                    code_location: self.get_pos(),
                                    note: None,
                                    help: None,
                                });
                                // return Err(format!("语法错误: 需要一个 /",));
                            }
                        }
                        x => {
                            self.push_and_advance(x);
                        }
                    }
                }
            }
        }

        {
            // 处理最后的状态
            if !self.current.is_empty() {
                match &self.state {
                    State::InIdentifier => {
                        self.mark_end_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::Identifier(std::mem::take(&mut self.current)).handle_keyword(),
                            loc,
                            self.gen_token_id(),
                        ));

                        self.current.clear();
                    }
                    State::InNumber => {
                        self.mark_end_pos();
                        let loc = self.get_pos();

                        // let num =
                        //     self.current
                        //         .parse()
                        //         .map_err(|e: ParseFloatError| ParserError {
                        //             file_path: self.file_path.clone(),
                        //             msg: e.to_string(),
                        //             code_location: self.get_pos(),
                        //             note: None,
                        //             help: None,
                        //         })?;
                        let num: f64 = match self.current.parse() {
                            Ok(f) => f,
                            Err(e) => {
                                errors.push(ParserError {
                                    file_path: self.file_path.clone(),
                                    msg: e.to_string(),
                                    code_location: self.get_pos(),
                                    note: None,
                                    help: None,
                                });

                                let re = LexerResult { tokens, errors };

                                return re;
                            }
                        };

                        tokens.push(Token::new(tk::Number(num), loc, self.gen_token_id()));

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
                            // let v = u64::from_str_radix(&self.current[2..], 16).map_err(|e| {
                            //     ParserError {
                            //         note: None,
                            //         help: None,
                            //         file_path: self.file_path.clone(),
                            //         msg: e.to_string(),
                            //         code_location: self.get_pos(),
                            //     }
                            // })?;
                            let v = match u64::from_str_radix(&self.current[2..], 16) {
                                Ok(f) => f,
                                Err(e) => {
                                    errors.push(ParserError {
                                        note: None,
                                        help: None,
                                        file_path: self.file_path.clone(),
                                        msg: e.to_string(),
                                        code_location: self.get_pos(),
                                    });

                                    let re = LexerResult { tokens, errors };
                                    return re;
                                }
                            };

                            if 是负数吗 {
                                (v as f64) * (-1.0)
                            } else {
                                v as f64
                            }
                        };

                        self.mark_end_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::Number(hex_value as f64),
                            loc,
                            self.gen_token_id(),
                        ));

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

                        // let binary_value =
                        //     u64::from_str_radix(&self.current[2..], 2).map_err(|e| {
                        //         ParserError {
                        //             note: None,
                        //             help: None,
                        //             file_path: self.file_path.clone(),
                        //             msg: e.to_string(),
                        //             code_location: self.get_pos(),
                        //         }
                        //     })?;
                        let binary_value = match u64::from_str_radix(&self.current[2..], 2) {
                            Ok(f) => f,
                            Err(e) => {
                                errors.push(ParserError {
                                    note: None,
                                    help: None,
                                    file_path: self.file_path.clone(),
                                    msg: e.to_string(),
                                    code_location: self.get_pos(),
                                });
                                let re = LexerResult { tokens, errors };
                                return re;
                            }
                        };

                        let binary_value = if 是负数吗 {
                            (binary_value as f64) * (-1.0)
                        } else {
                            binary_value as f64
                        };

                        self.mark_end_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::Number(binary_value as f64),
                            loc,
                            self.gen_token_id(),
                        ));

                        self.current.clear();
                    }

                    State::InLineComment => {
                        self.mark_end_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::LineComment(std::mem::take(&mut self.current)),
                            loc,
                            self.gen_token_id(),
                        ));
                    }
                    State::Initial => {
                        #[cfg(debug_assertions)]
                        todo!();
                    }
                    State::InString => {
                        // string ends.
                        // self.push(ch);
                        // self.advance(); // eat this character

                        self.mark_end_pos();
                        // self.advance();

                        let loc = self.get_pos();

                        // self.current 存放 String 字面量的内容.

                        tokens.push(Token::new(
                            tk::String(std::mem::take(&mut self.current)),
                            loc,
                            self.gen_token_id(),
                        ));

                        self.state = State::Initial;
                        self.current.clear();
                    }
                    State::InDocComment => {
                        self.mark_end_pos();

                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            // tk::DocComment(std::mem::take(&mut self.current)),
                            tk::DocComment(self.current.clone()),
                            loc,
                            self.gen_token_id(),
                        ));

                        self.state = State::Initial;
                        self.current.clear();
                    }
                    State::InBlockComment => {
                        // 结束注释

                        self.mark_end_pos();
                        let loc = self.get_pos();

                        tokens.push(Token::new(
                            tk::BlockComment(std::mem::take(&mut self.current)),
                            loc,
                            self.gen_token_id(),
                        ));

                        self.state = State::Initial;
                        self.current.clear();
                    }
                }
            }
        }

        let re = LexerResult { tokens, errors };
        return re;
    }
}

impl Lexer {
    fn get_current_position(&self) -> Position {
        Position::new(self.line as u32, self.column as u32, self.position)
    }

    fn get_pos(&mut self) -> Span {
        let start = match self.start_pos.clone() {
            Some(p) => p,
            None => {
                #[cfg(debug_assertions)]
                panic!("self.start_pos is None {:?}", self);

                #[allow(unreachable_code)]
                self.get_current_position()
            }
        };
        let end = match self.end_pos.clone() {
            Some(p) => p,
            None => self.get_current_position(),
        };

        self.start_pos = None;
        self.end_pos = None;

        return Span { start, end };
    }

    fn mark_start_pos(&mut self) {
        self.start_pos = self.get_current_position().into();
        self.end_pos = self.start_pos.clone();
    }

    fn mark_end_pos(&mut self) {
        if self.start_pos.is_none() {
            self.start_pos = self.get_current_position().into();
        }
        self.end_pos = self.get_current_position().into();
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod test {

    use crate::dp;

    use super::*;

    #[test]
    fn sadf() {
        sadfdasf(EXMAPLE_1);
        // sadfdasf(exmaple_2);
    }

    fn sadfdasf(code: &str) {
        let a = Lexer::new("file_path", code).tokenize();

        for token in a.tokens {
            dp(format!("{:?}", token.kind));
        }

        dp(format!("Error: {:?}", a.errors));
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
