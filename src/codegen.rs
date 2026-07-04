use crate::ast::*;
use std::collections::HashMap;

pub struct CodeGen {
    output: String,
    /// Variable name -> offset from frame pointer (fp, i.e., s0).
    /// Offset is negative: saved ra at fp-8, saved fp at fp-16,
    /// first local at fp-24, second at fp-32, etc.
    locals: HashMap<String, i64>,
    /// Next free offset (grows downward from fp).
    next_offset: i64,
    /// Frame size for current function (16-byte aligned).
    frame_size: i64,
    /// Label counter for unique jump labels.
    lbl: u64,
    /// Current function name (for epilogue labels).
    cur_func: String,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            output: String::new(),
            locals: HashMap::new(),
            next_offset: -24,
            frame_size: 0,
            lbl: 0,
            cur_func: String::new(),
        }
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn new_label(&mut self) -> String {
        let l = format!(".L{}", self.lbl);
        self.lbl += 1;
        l
    }

    // ── top-level ──────────────────────────────────────────

    pub fn generate(&mut self, program: &Program) -> String {
        self.emit(".text");

        for func in &program.functions {
            self.compile_function(func);
        }

        // Entry point
        self.emit("");
        self.emit(".globl _start");
        self.emit("_start:");
        self.emit("    call main");
        self.emit("    li a7, 93        # __NR_exit");
        self.emit("    ecall");

        self.output.clone()
    }

    // ── function compilation ──────────────────────────────

    fn compile_function(&mut self, func: &Function) {
        self.locals.clear();
        self.next_offset = -24;
        self.lbl = 0;
        self.cur_func = func.name.clone();

        // (1) Bind every declared variable to a stack slot.
        //     Parameters come first (they were injected as Decl stmts
        //     at the top of the body by the parser).
        let param_count = func.param_count;
        self.bind_vars(&func.body);

        // (2) Compute frame size.
        let local_count = func.body.iter().filter(|s| matches!(s, Stmt::Decl(..))).count() as i64;
        let needed = 16 + local_count * 8;
        self.frame_size = ((needed + 15) / 16) * 16;

        // (3) Emit label + prologue.
        self.emit("");
        self.emit(&format!(".globl {}", func.name));
        self.emit(&format!("{}:", func.name));
        self.emit(&format!("    # prologue (frame {} B)", self.frame_size));
        self.emit(&format!("    addi sp, sp, -{}", self.frame_size));
        self.emit(&format!("    sd ra, {}(sp)", self.frame_size - 8));
        self.emit(&format!("    sd fp, {}(sp)", self.frame_size - 16));
        self.emit(&format!("    addi fp, sp, {}   # fp = original sp", self.frame_size));

        // (4) Store incoming parameters into their slots.
        self.emit("");
        self.emit("    # store parameters");
        let regs = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];
        let mut pi: usize = 0;
        for stmt in &func.body {
            if let Stmt::Decl(name, _) = stmt {
                if pi < param_count && pi < 8 {
                    let off = self.locals[name];
                    self.emit(&format!("    sd {}, {}(fp)", regs[pi], off));
                }
                pi += 1;
                if pi >= param_count {
                    break;
                }
            }
        }

        // (5) Emit body statements (skip parameter Decl stmts).
        self.emit("");
        self.emit("    # body");
        let mut skip = param_count;
        for stmt in &func.body {
            if skip > 0 && let Stmt::Decl(_, _) = stmt {
                skip -= 1;
                continue;
            }
            self.gen_stmt(stmt);
        }

        // (6) Epilogue label (jumped to by return stmts).
        self.emit(&format!(".L_ret_{}:", self.cur_func));
        self.emit("    # epilogue");
        self.emit("    ld ra, -8(fp)");
        self.emit("    ld fp, -16(fp)");
        self.emit(&format!("    addi sp, sp, {}", self.frame_size));
        self.emit("    ret");
    }

    /// Walk declarations and assign stack offsets.
    fn bind_vars(&mut self, body: &[Stmt]) {
        for stmt in body {
            if let Stmt::Decl(name, _) = stmt
                && !self.locals.contains_key(name)
            {
                self.locals.insert(name.clone(), self.next_offset);
                self.next_offset -= 8;
            }
        }
    }

    // ── statements ─────────────────────────────────────────

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                self.gen_expr(e);
            }
            Stmt::Compound(ss) => {
                for s in ss {
                    self.gen_stmt(s);
                }
            }
            Stmt::If(cond, then, else_opt) => {
                let l_else = self.new_label();
                let l_end = self.new_label();

                self.gen_expr(cond);
                self.emit(&format!("    beqz a0, {}", l_else));
                self.gen_stmt(then);
                self.emit(&format!("    j {}", l_end));
                self.emit(&format!("{}:", l_else));
                if let Some(es) = else_opt {
                    self.gen_stmt(es);
                }
                self.emit(&format!("{}:", l_end));
            }
            Stmt::While(cond, body) => {
                let l_start = self.new_label();
                let l_end = self.new_label();

                self.emit(&format!("{}:", l_start));
                self.gen_expr(cond);
                self.emit(&format!("    beqz a0, {}", l_end));
                self.gen_stmt(body);
                self.emit(&format!("    j {}", l_start));
                self.emit(&format!("{}:", l_end));
            }
            Stmt::Return(e) => {
                self.gen_expr(e);
                self.emit(&format!("    j .L_ret_{}", self.cur_func));
            }
            Stmt::Decl(name, init) => {
                if let Some(expr) = init {
                    self.gen_expr(expr);
                    let off = self.locals[name.as_str()];
                    self.emit(&format!("    sd a0, {}(fp)", off));
                }
            }
        }
    }

    // ── expressions ────────────────────────────────────────
    //
    // Every expression compiles to code that leaves its result in a0.
    // The stack is used as scratch space for intermediate values.
    //

    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLit(n) => {
                self.emit(&format!("    li a0, {}", n));
            }
            Expr::StrLit(_s) => {
                // String literals are awkward in a single-pass codegen.
                // Emit a .rodata string and load its address.
                let label = format!(".LC{}", self.lbl);
                self.lbl += 1;
                self.emit(&format!("    la a0, {}", label));
                // TODO: emit string data in .rodata, but since we're emitting
                // to .text, this is just a placeholder. For now strings
                // are only used in test harnesses.
            }
            Expr::Ident(name) => {
                let off = self.locals.get(name.as_str())
                    .unwrap_or_else(|| panic!("undefined variable: {}", name));
                self.emit(&format!("    ld a0, {}(fp)", off));
            }

            // ── binary ops ────────────────────────────────
            Expr::Binary(left, op, right) => {
                match op {
                    BinOp::And => self.gen_logical_and(left, right),
                    BinOp::Or  => self.gen_logical_or(left, right),
                    _          => self.gen_arith_binary(left, op, right),
                }
            }

            // ── unary ops ────────────────────────────────
            Expr::Unary(op, inner) => {
                self.gen_expr(inner);
                match op {
                    UnaryOp::Neg    => self.emit("    neg a0, a0"),
                    UnaryOp::Not    => self.emit("    seqz a0, a0"),
                    UnaryOp::BitNot => self.emit("    not a0, a0"),
                }
            }

            // ── assignment ────────────────────────────────
            Expr::Assign(lhs, rhs) => {
                // Evaluate rhs, save it
                self.gen_expr(rhs);
                self.emit("    addi sp, sp, -8");
                self.emit("    sd a0, 0(sp)");

                // Compute lhs address into t0
                match lhs.as_ref() {
                    Expr::Ident(name) => {
                        let off = self.locals.get(name.as_str())
                            .unwrap_or_else(|| panic!("undefined: {}", name));
                        self.emit(&format!("    addi t0, fp, {}", off));
                    }
                    Expr::Deref(inner) => {
                        self.gen_expr(inner);
                        self.emit("    mv t0, a0");
                    }
                    _ => {
                        // Invalid lvalue — ignore for now
                    }
                }

                // Restore rhs, store
                self.emit("    ld a0, 0(sp)");
                self.emit("    addi sp, sp, 8");
                self.emit("    sd a0, 0(t0)");
            }

            // ── function call ─────────────────────────────
            Expr::Call(name, args) => self.gen_call(name, args),

            // ── address-of / deref (also handled in unary) ─
            Expr::AddrOf(inner) => {
                // Reuse unary logic
                if let Expr::Ident(name) = inner.as_ref() {
                    let off = self.locals.get(name.as_str())
                        .unwrap_or_else(|| panic!("undefined: {}", name));
                    self.emit(&format!("    addi a0, fp, {}", off));
                } else {
                    self.gen_expr(inner);
                }
            }
            Expr::Deref(inner) => {
                self.gen_expr(inner);
                self.emit("    ld a0, 0(a0)");
            }
        }
    }

    // ── helpers ──────────────────────────────────────────────

    fn gen_arith_binary(&mut self, left: &Expr, op: &BinOp, right: &Expr) {
        // eval left → push
        self.gen_expr(left);
        self.emit("    addi sp, sp, -8");
        self.emit("    sd a0, 0(sp)");

        // eval right → a0
        self.gen_expr(right);

        // pop left → t0
        self.emit("    ld t0, 0(sp)");
        self.emit("    addi sp, sp, 8");

        match op {
            BinOp::Add => self.emit("    add a0, t0, a0"),
            BinOp::Sub => self.emit("    sub a0, t0, a0"),
            BinOp::Mul => self.emit("    mul a0, t0, a0"),
            BinOp::Div => self.emit("    div a0, t0, a0"),
            BinOp::Mod => self.emit("    rem a0, t0, a0"),

            BinOp::Eq => {
                self.emit("    sub t0, t0, a0");
                self.emit("    seqz a0, t0");
            }
            BinOp::Ne => {
                self.emit("    sub t0, t0, a0");
                self.emit("    snez a0, t0");
            }
            BinOp::Lt => self.emit("    slt a0, t0, a0"),
            BinOp::Gt => self.emit("    slt a0, a0, t0"),
            BinOp::Le => {
                // left <= right  ⇔  !(right < left)
                self.emit("    slt a0, a0, t0");
                self.emit("    xori a0, a0, 1");
            }
            BinOp::Ge => {
                // left >= right  ⇔  !(left < right)
                self.emit("    slt a0, t0, a0");
                self.emit("    xori a0, a0, 1");
            }
            _ => unreachable!(),
        }
    }

    fn gen_logical_and(&mut self, left: &Expr, right: &Expr) {
        let l_false = self.new_label();
        let l_end = self.new_label();

        self.gen_expr(left);
        self.emit(&format!("    beqz a0, {}", l_false));
        self.gen_expr(right);
        self.emit("    snez a0, a0");
        self.emit(&format!("    j {}", l_end));
        self.emit(&format!("{}:", l_false));
        self.emit("    li a0, 0");
        self.emit(&format!("{}:", l_end));
    }

    fn gen_logical_or(&mut self, left: &Expr, right: &Expr) {
        let l_true = self.new_label();
        let l_end = self.new_label();

        self.gen_expr(left);
        self.emit(&format!("    bnez a0, {}", l_true));
        self.gen_expr(right);
        self.emit("    snez a0, a0");
        self.emit(&format!("    j {}", l_end));
        self.emit(&format!("{}:", l_true));
        self.emit("    li a0, 1");
        self.emit(&format!("{}:", l_end));
    }

    fn gen_call(&mut self, name: &str, args: &[Expr]) {
        let n = args.len().min(8);

        // Evaluate args into temporaries t0–t5, and a6-a7 onto stack
        for (i, arg) in args.iter().enumerate() {
            if i >= 8 {
                break;
            }
            self.gen_expr(arg);
            match i {
                0 => self.emit("    mv t0, a0"),
                1 => self.emit("    mv t1, a0"),
                2 => self.emit("    mv t2, a0"),
                3 => self.emit("    mv t3, a0"),
                4 => self.emit("    mv t4, a0"),
                5 => self.emit("    mv t5, a0"),
                6 => {
                    self.emit("    addi sp, sp, -8");
                    self.emit("    sd a0, 0(sp)");
                }
                7 => {
                    self.emit("    addi sp, sp, -8");
                    self.emit("    sd a0, 0(sp)");
                }
                _ => {}
            }
        }

        // Move temps into argument registers
        match n {
            8 => {
                self.emit("    ld a7, 0(sp)");
                self.emit("    addi sp, sp, 8");
                self.emit("    ld a6, 0(sp)");
                self.emit("    addi sp, sp, 8");
                self.emit("    mv a5, t5");
                self.emit("    mv a4, t4");
                self.emit("    mv a3, t3");
                self.emit("    mv a2, t2");
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            7 => {
                self.emit("    ld a6, 0(sp)");
                self.emit("    addi sp, sp, 8");
                self.emit("    mv a5, t5");
                self.emit("    mv a4, t4");
                self.emit("    mv a3, t3");
                self.emit("    mv a2, t2");
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            6 => {
                self.emit("    mv a5, t5");
                self.emit("    mv a4, t4");
                self.emit("    mv a3, t3");
                self.emit("    mv a2, t2");
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            5 => {
                self.emit("    mv a4, t4");
                self.emit("    mv a3, t3");
                self.emit("    mv a2, t2");
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            4 => {
                self.emit("    mv a3, t3");
                self.emit("    mv a2, t2");
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            3 => {
                self.emit("    mv a2, t2");
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            2 => {
                self.emit("    mv a1, t1");
                self.emit("    mv a0, t0");
            }
            1 => {
                self.emit("    mv a0, t0");
            }
            _ => {}
        }

        self.emit(&format!("    call {}", name));
    }
}
