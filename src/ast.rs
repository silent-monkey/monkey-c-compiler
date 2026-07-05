use std::fmt;

/// Represents a Monkey C type.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
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
    /// Pointer type: mutable=true means *mut T, mutable=false means *const T
    Pointer {
        mutable: bool,
        inner: Box<Type>,
    },
}

impl Type {
    /// Size in bytes.
    pub fn size(&self) -> usize {
        match self {
            Type::Bool | Type::U8 | Type::I8 => 1,
            Type::U16 | Type::I16 => 2,
            Type::U32 | Type::I32 => 4,
            Type::U64 | Type::I64 | Type::Usize | Type::Isize => 8,
            Type::Pointer { .. } => 8,
        }
    }

    /// Alignment in bytes.
    pub fn align(&self) -> usize {
        self.size()
    }

    /// RISC-V store suffix (b, h, w, d).
    pub fn mem_suffix(&self) -> &'static str {
        match self.size() {
            1 => "b",
            2 => "h",
            4 => "w",
            8 => "d",
            _ => unreachable!(),
        }
    }

    /// RISC-V load suffix (unsigned loads use u suffix for sub-word sizes).
    pub fn load_suffix(&self, signed: bool) -> &'static str {
        match self.size() {
            1 => if signed { "b" } else { "bu" },
            2 => if signed { "h" } else { "hu" },
            4 => if signed { "w" } else { "wu" },
            8 => "d",
            _ => unreachable!(),
        }
    }

    /// Whether this is a signed integer type.
    pub fn is_signed(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Isize)
    }

    /// Whether this is an unsigned integer type.
    #[allow(dead_code)]
    pub fn is_unsigned(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Usize | Type::Bool)
    }

    /// Whether this is a pointer type.
    #[allow(dead_code)]
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer { .. })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::Usize => write!(f, "usize"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::Isize => write!(f, "isize"),
            Type::Pointer { mutable, inner } => {
                if *mutable {
                    write!(f, "*mut {}", inner)
                } else {
                    write!(f, "*const {}", inner)
                }
            }
        }
    }
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// Top-level program: a list of declarations.
#[derive(Debug, Clone)]
pub struct Program {
    pub decls: Vec<Decl>,
}

/// A top-level declaration.
#[derive(Debug, Clone)]
pub enum Decl {
    Function {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    #[allow(dead_code)]
    ExternFunction {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        has_varargs: bool,
    },
}

/// A statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Type,
        init: Option<Box<Expr>>,
    },
    Expr(Box<Expr>),
    Return(Option<Box<Expr>>),
    If {
        cond: Box<Expr>,
        then_body: Box<Stmt>,
        else_body: Option<Box<Stmt>>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Stmt>,
    },
    Loop {
        body: Box<Stmt>,
    },
    Break,
    Continue,
    Block(Vec<Stmt>),
}

/// An expression.
#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    StringLiteral(String),
    Ident(String),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Assign {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Eq,
    NotEq,
    #[allow(dead_code)]
    And,
    #[allow(dead_code)]
    Or,
}

impl BinaryOp {
    #[allow(dead_code)]
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq | BinaryOp::Eq | BinaryOp::NotEq
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    Addr,
    Deref,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignOp {
    Simple,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}
