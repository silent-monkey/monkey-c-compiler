use std::str::Chars;

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Extern,
    Fn,
    Let,
    If,
    While,
    Loop,
    Break,
    Continue,
    Const,
    Else,
    Mut,
    Return,
    // Types
    Bool,
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
    // Symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Colon,
    Comma,
    Arrow,     // ->
    Star,      // *
    DotDotDot, // ...
    Plus,
    Minus,
    Slash,
    Percent,
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    #[allow(dead_code)]
    SlashEq,
    PercentEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    EqEq,
    NotEq,
    Ampersand,
    Exclaim,
    // Literals
    IntLiteral(i64),
    StringLiteral(String),
    // Other
    Identifier(String),
    Eof,
}

impl TokenKind {
    #[allow(dead_code)]
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            TokenKind::Bool
                | TokenKind::U8
                | TokenKind::U16
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::Usize
                | TokenKind::I8
                | TokenKind::I16
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::Isize
        )
    }
}

pub struct Lexer<'a> {
    chars: Chars<'a>,
    peeked: Option<char>,
    line: usize,
    col: usize,
    #[allow(dead_code)]
    source: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let peeked = chars.next();
        Lexer {
            chars,
            peeked,
            line: 1,
            col: 1,
            source,
            pos: 0,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peeked;
        if let Some(ch) = c {
            self.pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.peeked = self.chars.next();
        c
    }

    fn peek(&self) -> Option<char> {
        self.peeked
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_number(&mut self, first: char) -> TokenKind {
        let mut num_str = String::new();
        num_str.push(first);
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let val: i64 = num_str.parse().expect("invalid integer literal");
        TokenKind::IntLiteral(val)
    }

    fn read_identifier(&mut self, first: char) -> TokenKind {
        let mut ident = String::new();
        ident.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match ident.as_str() {
            "extern" => TokenKind::Extern,
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "const" => TokenKind::Const,
            "else" => TokenKind::Else,
            "mut" => TokenKind::Mut,
            "return" => TokenKind::Return,
            "bool" => TokenKind::Bool,
            "u8" => TokenKind::U8,
            "u16" => TokenKind::U16,
            "u32" => TokenKind::U32,
            "u64" => TokenKind::U64,
            "usize" => TokenKind::Usize,
            "i8" => TokenKind::I8,
            "i16" => TokenKind::I16,
            "i32" => TokenKind::I32,
            "i64" => TokenKind::I64,
            "isize" => TokenKind::Isize,
            _ => TokenKind::Identifier(ident),
        }
    }

    fn read_string(&mut self) -> TokenKind {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance(); // consume closing "
                break;
            } else if c == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        s.push('\n');
                        self.advance();
                    }
                    Some('t') => {
                        s.push('\t');
                        self.advance();
                    }
                    Some('\\') => {
                        s.push('\\');
                        self.advance();
                    }
                    Some('"') => {
                        s.push('"');
                        self.advance();
                    }
                    Some('0') => {
                        s.push('\0');
                        self.advance();
                    }
                    Some(c) => {
                        s.push('\\');
                        s.push(c);
                        self.advance();
                    }
                    None => break,
                }
            } else {
                s.push(c);
                self.advance();
            }
        }
        TokenKind::StringLiteral(s)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let line = self.line;
        let col = self.col;

        let kind = match self.advance() {
            None => TokenKind::Eof,

            // Line comment
            Some('/') if self.peek() == Some('/') => {
                self.skip_line_comment();
                return self.next_token(); // recurse
            }
            Some('/') => TokenKind::Slash,

            Some('(') => TokenKind::LParen,
            Some(')') => TokenKind::RParen,
            Some('{') => TokenKind::LBrace,
            Some('}') => TokenKind::RBrace,
            Some(';') => TokenKind::Semicolon,
            Some(':') => TokenKind::Colon,
            Some(',') => TokenKind::Comma,
            Some('&') => TokenKind::Ampersand,

            Some('*') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::StarEq
            }
            Some('*') => TokenKind::Star,

            Some('+') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::PlusEq
            }
            Some('+') => TokenKind::Plus,

            Some('-') if self.peek() == Some('>') => {
                self.advance();
                TokenKind::Arrow
            }
            Some('-') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::MinusEq
            }
            Some('-') => TokenKind::Minus,

            Some('%') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::PercentEq
            }
            Some('%') => TokenKind::Percent,

            Some('=') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::EqEq
            }
            Some('=') => TokenKind::Eq,

            Some('!') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::NotEq
            }
            Some('!') => TokenKind::Exclaim,

            Some('<') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::LtEq
            }
            Some('<') => TokenKind::Lt,

            Some('>') if self.peek() == Some('=') => {
                self.advance();
                TokenKind::GtEq
            }
            Some('>') => TokenKind::Gt,

            Some('.') if self.peek() == Some('.') => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    TokenKind::DotDotDot
                } else {
                    // Unexpected: just return Dot for now (shouldn't happen)
                    TokenKind::Eof
                }
            }

            Some('"') => self.read_string(),

            Some(c) if c.is_ascii_digit() => self.read_number(c),

            Some(c) if c.is_alphabetic() || c == '_' => self.read_identifier(c),

            Some(c) => {
                // Unknown character, skip it but report an error later if needed
                eprintln!(
                    "Warning: skipping unknown character '{}' at {}:{}",
                    c, line, col
                );
                return self.next_token();
            }
        };

        Token { kind, line, col }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
