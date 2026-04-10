use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Token {
    // Special
    Illegal(char),
    Eof,

    // Identifiers + literals
    Ident(String),
    Int(String),
    Str(String),

    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Eq,
    NotEq,
    And,
    Or,
    Conditional,
    // Compound assignment operators
    PlusAssign,     // +=
    MinusAssign,    // -=
    AsteriskAssign, // *=
    SlashAssign,    // /=

    // Delimiters
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LSquare,
    RSquare,
    Colon,
    Dot,

    // Keywords
    Function,
    For,
    Let,
    True,
    False,
    If,
    Else,
    Return,
    Break,
    Continue,
}

impl Token {
    pub fn to_string(&self) -> String {
        match self {
            Token::Illegal(c) => c.to_string(),
            Token::Eof => "EOF".to_string(),
            Token::Ident(s) => s.clone(),
            Token::Int(s) => s.clone(),
            Token::Str(s) => format!("\"{}\"", s),
            Token::Assign => "=".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Bang => "!".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Lt => "<".to_string(),
            Token::Gt => ">".to_string(),
            Token::Eq => "==".to_string(),
            Token::NotEq => "!=".to_string(),
            Token::And => "&&".to_string(),
            Token::Or => "||".to_string(),
            Token::PlusAssign => "+=".to_string(),
            Token::MinusAssign => "-=".to_string(),
            Token::AsteriskAssign => "*=".to_string(),
            Token::SlashAssign => "/=".to_string(),
            Token::Comma => ",".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
            Token::LBrace => "{".to_string(),
            Token::RBrace => "}".to_string(),
            Token::Function => "func".to_string(),
            Token::For => "for".to_string(),
            Token::Let => "let".to_string(),
            Token::True => "true".to_string(),
            Token::False => "false".to_string(),
            Token::If => "if".to_string(),
            Token::Else => "else".to_string(),
            Token::Return => "return".to_string(),
            Token::LSquare => "[".to_string(),
            Token::RSquare => "]".to_string(),
            Token::Break => "break".to_string(),
            Token::Continue => "continue".to_string(),
            Token::Colon => ":".to_string(),
            Token::Dot => ".".to_string(),
            Token::Conditional => "?".to_string(),
        }
    }
}

pub fn lookup_identifier(ident: &str) -> Token {
    match ident {
        "func" => Token::Function,
        "let" => Token::Let,
        "true" => Token::True,
        "false" => Token::False,
        "if" => Token::If,
        "else" => Token::Else,
        "return" => Token::Return,
        "for" => Token::For,
        "break" => Token::Break,
        "continue" => Token::Continue,
        _ => Token::Ident(ident.to_string()),
    }
}
