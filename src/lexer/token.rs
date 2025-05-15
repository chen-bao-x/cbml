#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct TokenID(pub(crate) u64);

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,

    /// 只读
    token_id: TokenID,
}

impl Token {
    pub fn new(kind: TokenKind, location: Span, token_id: TokenID) -> Self {
        Token {
            kind,
            span: location,
            token_id,
        }
    }

    pub fn get_id(&self) -> TokenID {
        self.token_id
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
    DoubleQuote,  // "

    // key words
    True,    // true
    False,   // false
    TkNone,  // none
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

    #[allow(dead_code)]
    Invalid(char), // invalid character

    EOF, // end of file
}

impl TokenKind {
    pub fn handle_keyword(&self) -> Self {
        match self {
            TokenKind::Identifier(s) => match s.as_str() {
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "none" => TokenKind::TkNone,
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
            (TokenKind::TkNone, TokenKind::TkNone) => true,
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
            (TokenKind::DoubleQuote, TokenKind::DoubleQuote) => true,

            _ => false,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn lookup<'a>(&self, code: &'a str) -> Option<&'a str> {
        let start_pos = self.start.character_index;
        let end_pos = self.end.character_index;
        code.get(start_pos..=end_pos)
    }

    pub fn empty() -> Self {
        Self {
            start: Position {
                line: 0,
                column: 0,
                character_index: 0,
            },
            end: Position {
                line: 0,
                column: 0,
                character_index: 0,
            },
        }
    }

    pub fn is_contain(&self, line: u32, colunm: u32) -> bool {
        let a = self.start.line <= line && self.end.line >= line;
        let b = self.start.column <= colunm && self.end.column >= colunm;

        return a && b;
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// 行号, 最开头的一个是 0,
    // pub line: usize,
    pub line: u32,

    /// 列号, 最开头的一个是 0,
    pub column: u32,

    /// 在文本中的 index.
    pub character_index: usize,
}

impl Position {
    pub fn new(line: u32, column: u32, character_index: usize) -> Self {
        Self {
            line,
            column,
            character_index,
        }
    }
}
