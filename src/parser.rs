use crate::ast::*;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        let t = self.peek();
        if t.kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "expected {:?} but got {:?} at position {}",
                kind,
                t.kind,
                t.pos
            ))
        }
    }

    fn is_type(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Int)
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        while self.peek().kind != TokenKind::Eof {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        // int name(...)
        self.expect(TokenKind::Int)?;
        let name = match &self.peek().kind {
            TokenKind::Ident(s) => s.clone(),
            _ => return Err("expected function name".to_string()),
        };
        self.advance();

        // Parameter list (must contain parameter declarations)
        self.expect(TokenKind::LParen)?;
        let mut params: Vec<(String, bool)> = Vec::new();
        if self.is_type() {
            loop {
                self.expect(TokenKind::Int)?;
                let pname = match &self.peek().kind {
                    TokenKind::Ident(s) => s.clone(),
                    _ => return Err("expected parameter name".to_string()),
                };
                self.advance();
                params.push((pname, true));
                if self.peek().kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;

        let param_count = params.len();

        // Body
        let body = self.parse_compound_statement(params)?;

        Ok(Function { name, param_count, body })
    }

    fn parse_compound_statement(&mut self, params: Vec<(String, bool)>) -> Result<Vec<Stmt>, String> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();

        // Add parameter declarations as local variables
        // We mark them as initialized since they come from the caller
        for (name, initialized) in &params {
            stmts.push(Stmt::Decl(name.clone(), if *initialized { Some(Expr::IntLit(0)) } else { None }));
        }

        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::Eof {
            stmts.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.peek().kind {
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => self.parse_return(),
            TokenKind::LBrace => self.parse_compound_statement(Vec::new()).map(Stmt::Compound),
            TokenKind::Int => self.parse_declaration(),
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_declaration(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Int)?;
        let name = match &self.peek().kind {
            TokenKind::Ident(s) => s.clone(),
            _ => return Err("expected variable name".to_string()),
        };
        self.advance();

        let init = if self.peek().kind == TokenKind::Assign {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Decl(name, init))
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.peek().kind == TokenKind::Else {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Stmt::If(cond, then_branch, else_branch))
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::While)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::While(cond, body))
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Return)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_expr_statement(&mut self) -> Result<Stmt, String> {
        if self.peek().kind == TokenKind::Semicolon {
            self.advance();
            return Ok(Stmt::Expr(Expr::IntLit(0))); // empty statement
        }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_or()?;
        if self.peek().kind == TokenKind::Assign {
            self.advance();
            let right = self.parse_assignment()?;
            Ok(Expr::Assign(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek().kind == TokenKind::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.peek().kind == TokenKind::And {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_relational()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Eq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek().kind {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
            }
            TokenKind::Ampersand => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::AddrOf(Box::new(expr)))
            }
            TokenKind::Star => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Deref(Box::new(expr)))
            }
            TokenKind::BitNot => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::BitNot, Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek().kind == TokenKind::LParen {
                // Function call
                self.advance();
                let mut args = Vec::new();
                if self.peek().kind != TokenKind::RParen {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen)?;
                match &expr {
                    Expr::Ident(name) => {
                        expr = Expr::Call(name.clone(), args);
                    }
                    _ => return Err("expected function name before '('".to_string()),
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match &self.peek().kind {
            TokenKind::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::IntLit(val))
            }
            TokenKind::StrLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::StrLit(val))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(format!("unexpected token {:?} at position {}", self.peek().kind, self.peek().pos)),
        }
    }
}
