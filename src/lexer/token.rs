#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Token { kind, line, column }
    }
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({:?}, {}, {})", self.kind, self.line, self.column)
    }
}
impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind.kind_is(&other.kind)
    }
}
// impl std::cmp::Eq for Token {}

#[derive(Clone, Debug)]
pub enum TokenKind {
    String(String), // staring literal
    Number(f64),    // number literal 十进制 二进制 十六进制 写法.

    LineComment(String),
    BlockComment(String),
    DocComment(String),

    Identifier(String),

    LParen,       // (
    RParen,       // )
    LBracket,     // [
    RBracket,     // ]
    LBrace,       // {
    RBrace,       // }
    Comma,        // ,
    Colon,        // :
    Pipe,         // |
    QuestionMark, // ?
    Asign,        // =
    NewLine,      // new line

    // key words
    True,    // true
    False,   // false
    None,    // none
    Any,     // any
    Struct,  // struct
    Union,   // union
    Todo,    // todo
    Use,     // use
    Default, // default
    Enum,    // enum

    StringTy,  // bool
    NumberTy,  // number
    BooleanTy, // bool

    Invalid(char), // invalid character

    EOF, // end of file
}

impl TokenKind {
    pub fn handle_keyword(&self) -> Self {
        match self {
            TokenKind::Identifier(s) => match s.as_str() {
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "none" => TokenKind::None,
                "any" => TokenKind::Any,
                "struct" => TokenKind::Struct,
                "union" => TokenKind::Union,
                "todo" => TokenKind::Todo,
                "use" => TokenKind::Use,
                "default" => TokenKind::Default,
                "enum" => TokenKind::Enum,

                "string" => TokenKind::StringTy,
                "number" => TokenKind::NumberTy,
                "bool" => TokenKind::BooleanTy,
                _ => TokenKind::Identifier(s.clone()),
            },
            _ => self.clone(),
        }
    }
    pub fn kind_is(&self, other: &Self) -> bool {
        match (self, other) {
            (TokenKind::String(_), TokenKind::String(_)) => true,
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::Invalid(_), TokenKind::Invalid(_)) => true,
            (TokenKind::LineComment(_), TokenKind::LineComment(_)) => true,
            (TokenKind::BlockComment(_), TokenKind::BlockComment(_)) => true,
            (TokenKind::DocComment(_), TokenKind::DocComment(_)) => true,
            (TokenKind::LParen, TokenKind::LParen) => true,
            (TokenKind::RParen, TokenKind::RParen) => true,
            (TokenKind::LBracket, TokenKind::LBracket) => true,
            (TokenKind::RBracket, TokenKind::RBracket) => true,
            (TokenKind::LBrace, TokenKind::LBrace) => true,
            (TokenKind::RBrace, TokenKind::RBrace) => true,
            (TokenKind::Comma, TokenKind::Comma) => true,
            (TokenKind::Colon, TokenKind::Colon) => true,
            (TokenKind::Pipe, TokenKind::Pipe) => true,
            (TokenKind::QuestionMark, TokenKind::QuestionMark) => true,
            (TokenKind::Asign, TokenKind::Asign) => true,
            (TokenKind::NewLine, TokenKind::NewLine) => true,
            (TokenKind::True, TokenKind::True) => true,
            (TokenKind::False, TokenKind::False) => true,
            (TokenKind::None, TokenKind::None) => true,
            (TokenKind::Any, TokenKind::Any) => true,
            (TokenKind::Struct, TokenKind::Struct) => true,
            (TokenKind::Union, TokenKind::Union) => true,
            (TokenKind::Todo, TokenKind::Todo) => true,
            (TokenKind::Use, TokenKind::Use) => true,
            (TokenKind::Default, TokenKind::Default) => true,
            (TokenKind::StringTy, TokenKind::StringTy) => true,
            (TokenKind::NumberTy, TokenKind::NumberTy) => true,
            (TokenKind::BooleanTy, TokenKind::BooleanTy) => true,
            (TokenKind::Enum, TokenKind::Enum) => true,
            (TokenKind::EOF, TokenKind::EOF) => true,
              
            _ => false,
        }
    }
}
