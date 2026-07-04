#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Int,
    If,
    Else,
    While,
    Return,

    // Identifiers and literals
    Ident(String),
    IntLit(i64),
    StrLit(String),

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Eq,         // ==
    Ne,         // !=
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=
    Assign,     // =
    Not,        // !
    And,        // &&
    Or,         // ||
    Ampersand,  // &
    BitNot,     // ~

    // Punctuation
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    Semicolon, // ;
    Comma,     // ,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}
