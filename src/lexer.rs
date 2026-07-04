use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            input: source.chars().collect(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek() == Some('/') {
                // Line comment
                self.advance();
                self.advance();
                while let Some(c) = self.current() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if ch == '/' && self.peek() == Some('*') {
                // Block comment
                self.advance();
                self.advance();
                loop {
                    match self.current() {
                        None => break,
                        Some('*') if self.peek() == Some('/') => {
                            self.advance();
                            self.advance();
                            break;
                        }
                        _ => self.advance(),
                    }
                }
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos + 1 < self.input.len() {
            Some(self.input[self.pos + 1])
        } else {
            None
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn read_number(&mut self) -> i64 {
        let mut s = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        s.parse().unwrap_or(0)
    }

    fn read_string(&mut self) -> Result<String, String> {
        let mut s = String::new();
        self.advance(); // skip opening "
        loop {
            match self.current() {
                None => return Err("unterminated string literal".to_string()),
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some(c) => s.push(c),
                        None => return Err("unterminated escape in string literal".to_string()),
                    }
                    self.advance();
                }
                Some(ch) => {
                    s.push(ch);
                    self.advance();
                }
            }
        }
        Ok(s)
    }

    fn keyword_or_ident(&self, s: &str) -> TokenKind {
        match s {
            "int" => TokenKind::Int,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            _ => TokenKind::Ident(s.to_string()),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        let pos = self.pos;

        match self.current() {
            None => Ok(Token { kind: TokenKind::Eof, pos }),
            Some('(') => { self.advance(); Ok(Token { kind: TokenKind::LParen, pos }) }
            Some(')') => { self.advance(); Ok(Token { kind: TokenKind::RParen, pos }) }
            Some('{') => { self.advance(); Ok(Token { kind: TokenKind::LBrace, pos }) }
            Some('}') => { self.advance(); Ok(Token { kind: TokenKind::RBrace, pos }) }
            Some(';') => { self.advance(); Ok(Token { kind: TokenKind::Semicolon, pos }) }
            Some(',') => { self.advance(); Ok(Token { kind: TokenKind::Comma, pos }) }
            Some('+') => { self.advance(); Ok(Token { kind: TokenKind::Plus, pos }) }
            Some('-') => { self.advance(); Ok(Token { kind: TokenKind::Minus, pos }) }
            Some('*') => { self.advance(); Ok(Token { kind: TokenKind::Star, pos }) }
            Some('/') => { self.advance(); Ok(Token { kind: TokenKind::Slash, pos }) }
            Some('%') => { self.advance(); Ok(Token { kind: TokenKind::Percent, pos }) }
            Some('~') => { self.advance(); Ok(Token { kind: TokenKind::BitNot, pos }) }
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    Ok(Token { kind: TokenKind::And, pos })
                } else {
                    Ok(Token { kind: TokenKind::Ampersand, pos })
                }
            }
            Some('|') => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    Ok(Token { kind: TokenKind::Or, pos })
                } else {
                    Err(format!("unexpected character '|' at position {}", pos))
                }
            }
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::Eq, pos })
                } else {
                    Ok(Token { kind: TokenKind::Assign, pos })
                }
            }
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::Ne, pos })
                } else {
                    Ok(Token { kind: TokenKind::Not, pos })
                }
            }
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::Le, pos })
                } else {
                    Ok(Token { kind: TokenKind::Lt, pos })
                }
            }
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::Ge, pos })
                } else {
                    Ok(Token { kind: TokenKind::Gt, pos })
                }
            }
            Some('"') => {
                let s = self.read_string()?;
                Ok(Token { kind: TokenKind::StrLit(s), pos })
            }
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                let s = self.read_ident();
                let kind = self.keyword_or_ident(&s);
                Ok(Token { kind, pos })
            }
            Some(ch) if ch.is_ascii_digit() => {
                let n = self.read_number();
                Ok(Token { kind: TokenKind::IntLit(n), pos })
            }
            Some(ch) => {
                Err(format!("unexpected character '{}' at position {}", ch, pos))
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}
