use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<String>,
}

/// Helper to check if two TokenKinds have the same discriminant (variant).
fn kind_eq(a: &TokenKind, b: &TokenKind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), String> {
        let t = self.advance();
        if kind_eq(&t.kind, &expected) {
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, got {:?} at line {} col {}",
                expected, t.kind, t.line, t.col
            ))
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        kind_eq(self.peek_kind(), kind)
    }

    fn is_eof(&self) -> bool {
        self.check(&TokenKind::Eof)
    }

    pub fn parse_program(&mut self) -> Program {
        let mut decls = Vec::new();
        while !self.is_eof() {
            match self.parse_decl() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        Program { decls }
    }

    fn synchronize(&mut self) {
        while !self.is_eof() {
            match self.peek_kind() {
                TokenKind::Fn | TokenKind::Extern => break,
                TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                TokenKind::Semicolon => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // ---- Declarations ----

    fn parse_decl(&mut self) -> Result<Decl, String> {
        match self.peek_kind() {
            TokenKind::Extern => self.parse_extern_decl(),
            TokenKind::Fn => self.parse_fn_decl(),
            _ => Err(format!(
                "Expected fn or extern, got {:?} at line {} col {}",
                self.peek_kind(),
                self.peek().line,
                self.peek().col
            )),
        }
    }

    fn parse_extern_decl(&mut self) -> Result<Decl, String> {
        self.expect(TokenKind::Extern)?;
        self.expect(TokenKind::Fn)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LParen)?;
        let (params, has_varargs) = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Decl::ExternFunction {
            name,
            params,
            return_type,
            has_varargs,
        })
    }

    fn parse_fn_decl(&mut self) -> Result<Decl, String> {
        self.expect(TokenKind::Fn)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LParen)?;
        let (params, _has_varargs) = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Decl::Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.advance().kind.clone() {
            TokenKind::Identifier(name) => Ok(name),
            other => Err(format!(
                "Expected identifier, got {:?} at line {} col {}",
                other, self.peek().line, self.peek().col
            )),
        }
    }

    fn parse_params(&mut self) -> Result<(Vec<Param>, bool), String> {
        let mut params = Vec::new();
        let mut has_varargs = false;

        if self.check(&TokenKind::RParen) {
            return Ok((params, has_varargs));
        }

        loop {
            if self.check(&TokenKind::DotDotDot) {
                self.advance();
                has_varargs = true;
                break;
            }

            let name = self.parse_identifier()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty });

            if self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::DotDotDot) {
                    self.advance();
                    has_varargs = true;
                    break;
                }
            } else {
                break;
            }
        }

        Ok((params, has_varargs))
    }

    // ---- Types ----

    fn parse_type(&mut self) -> Result<Type, String> {
        if self.check(&TokenKind::Star) {
            self.advance();
            let mutable = match self.peek_kind() {
                TokenKind::Const => {
                    self.advance();
                    false
                }
                TokenKind::Mut => {
                    self.advance();
                    true
                }
                other => {
                    return Err(format!(
                        "Expected const or mut after *, got {:?} at line {} col {}",
                        other, self.peek().line, self.peek().col
                    ));
                }
            };
            let inner = self.parse_type()?;
            Ok(Type::Pointer {
                mutable,
                inner: Box::new(inner),
            })
        } else {
            self.parse_base_type()
        }
    }

    fn parse_base_type(&mut self) -> Result<Type, String> {
        match self.advance().kind.clone() {
            TokenKind::Bool => Ok(Type::Bool),
            TokenKind::U8 => Ok(Type::U8),
            TokenKind::U16 => Ok(Type::U16),
            TokenKind::U32 => Ok(Type::U32),
            TokenKind::U64 => Ok(Type::U64),
            TokenKind::Usize => Ok(Type::Usize),
            TokenKind::I8 => Ok(Type::I8),
            TokenKind::I16 => Ok(Type::I16),
            TokenKind::I32 => Ok(Type::I32),
            TokenKind::I64 => Ok(Type::I64),
            TokenKind::Isize => Ok(Type::Isize),
            other => Err(format!(
                "Expected type, got {:?} at line {} col {}",
                other, self.peek().line, self.peek().col
            )),
        }
    }

    // ---- Statements ----

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            match self.parse_stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Loop => self.parse_loop_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                Ok(Stmt::Block(block))
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Let)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let init = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Let { name, ty, init })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::LParen)?;
        let cond = Box::new(self.parse_expr()?);
        self.expect(TokenKind::RParen)?;
        let then_body = Box::new(self.parse_stmt()?);
        let else_body = if self.check(&TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::While)?;
        self.expect(TokenKind::LParen)?;
        let cond = Box::new(self.parse_expr()?);
        self.expect(TokenKind::RParen)?;
        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::While { cond, body })
    }

    fn parse_loop_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Loop)?;
        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::Loop { body })
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Break)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Break)
    }

    fn parse_continue_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Continue)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Continue)
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Return)?;
        let expr = if !self.check(&TokenKind::Semicolon) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expr(Box::new(expr)))
    }

    // ---- Expressions ----

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_equality()?;

        let assign_op = match self.peek_kind() {
            TokenKind::Eq => Some(AssignOp::Simple),
            TokenKind::PlusEq => Some(AssignOp::Add),
            TokenKind::MinusEq => Some(AssignOp::Sub),
            TokenKind::StarEq => Some(AssignOp::Mul),
            TokenKind::SlashEq => Some(AssignOp::Div),
            TokenKind::PercentEq => Some(AssignOp::Rem),
            _ => None,
        };

        if let Some(op) = assign_op {
            self.advance();
            let right = Box::new(self.parse_assignment()?);
            Ok(Expr::Assign {
                target: Box::new(left),
                op,
                value: right,
            })
        } else {
            Ok(left)
        }
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_relational()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq => Some(BinaryOp::Eq),
                TokenKind::NotEq => Some(BinaryOp::NotEq),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = Box::new(self.parse_relational()?);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Lt => Some(BinaryOp::Lt),
                TokenKind::Gt => Some(BinaryOp::Gt),
                TokenKind::LtEq => Some(BinaryOp::LtEq),
                TokenKind::GtEq => Some(BinaryOp::GtEq),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = Box::new(self.parse_additive()?);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = Box::new(self.parse_multiplicative()?);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => Some(BinaryOp::Mul),
                TokenKind::Slash => Some(BinaryOp::Div),
                TokenKind::Percent => Some(BinaryOp::Rem),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = Box::new(self.parse_unary()?);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek_kind() {
            TokenKind::Minus => {
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    operand,
                })
            }
            TokenKind::Exclaim => {
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    operand,
                })
            }
            TokenKind::Ampersand => {
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::Addr,
                    operand,
                })
            }
            TokenKind::Star => {
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::Deref,
                    operand,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&TokenKind::LParen) {
                expr = self.parse_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_call(&mut self, callee: Expr) -> Result<Expr, String> {
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance().kind.clone() {
            TokenKind::IntLiteral(val) => Ok(Expr::IntLiteral(val)),
            TokenKind::StringLiteral(s) => Ok(Expr::StringLiteral(s)),
            TokenKind::Identifier(name) => Ok(Expr::Ident(name)),
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            other => Err(format!(
                "Expected expression, got {:?} at line {} col {}",
                other, self.peek().line, self.peek().col
            )),
        }
    }
}
