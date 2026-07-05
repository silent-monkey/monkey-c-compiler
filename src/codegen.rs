use crate::ast::*;
use std::collections::HashMap;

pub struct CodeGen {
    output: Vec<String>,
    strings: Vec<(String, String)>,
    label_counter: u64,
    string_counter: u64,
    locals: HashMap<String, (i64, Type)>,
    break_label: Vec<String>,
    continue_label: Vec<String>,
    temp_regs: Vec<&'static str>,
    next_temp: usize,
    loop_depth: usize,
    current_frame_size: usize,
    /// Base offset of spill area (from fp). Spill slots are at spill_base + i*8.
    spill_base: i64,
    /// Next available spill slot index.
    spill_next: usize,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            output: Vec::new(),
            strings: Vec::new(),
            label_counter: 0,
            string_counter: 0,
            locals: HashMap::new(),
            break_label: Vec::new(),
            continue_label: Vec::new(),
            temp_regs: vec!["t0", "t1", "t2", "t3", "t4", "t5", "t6"],
            next_temp: 0,
            loop_depth: 0,
            current_frame_size: 0,
            spill_base: 0,
            spill_next: 0,
        }
    }

    /// Spill a register value to the next available spill slot on stack.
    fn spill_reg(&mut self, reg: &str) -> usize {
        let idx = self.spill_next;
        self.spill_next += 1;
        let off = self.spill_base + (idx as i64) * 8;
        self.emit(&format!("    sd {}, {}(fp)", reg, off));
        idx
    }

    /// Load a value from a spill slot into a register.
    fn unspill_reg(&mut self, slot: usize) -> String {
        let r = self.alloc_temp().to_string();
        let off = self.spill_base + (slot as i64) * 8;
        self.emit(&format!("    ld {}, {}(fp)", r, off));
        r
    }

    fn fresh_label(&mut self) -> String {
        let l = format!(".L{}", self.label_counter);
        self.label_counter += 1;
        l
    }

    fn fresh_string_label(&mut self) -> String {
        let l = format!(".LC{}", self.string_counter);
        self.string_counter += 1;
        l
    }

    fn emit(&mut self, s: &str) {
        self.output.push(s.to_string());
    }

    fn alloc_temp(&mut self) -> &'static str {
        let r = self.temp_regs[self.next_temp];
        self.next_temp = (self.next_temp + 1) % self.temp_regs.len();
        r
    }

    fn reset_temps(&mut self) {
        self.next_temp = 0;
    }

    // ── Top-level generation ──

    pub fn generate(&mut self, program: &Program) -> String {
        self.emit("    .text");
        self.emit("");

        for decl in &program.decls {
            self.generate_decl(decl);
        }

        let mut result = Vec::new();

        // rodata section first
        if !self.strings.is_empty() {
            result.push("    .section .rodata".to_string());
            for (label, content) in &self.strings {
                let escaped = content
                    .replace("\\", "\\\\")
                    .replace("\n", "\\n")
                    .replace("\t", "\\t")
                    .replace("\"", "\\\"");
                result.push(format!("{}:", label));
                result.push(format!("    .string \"{}\"", escaped));
            }
            result.push(String::new());
        }

        result.extend(self.output.clone());
        result.join("\n")
    }

    fn generate_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function {
                name,
                params,
                return_type,
                body,
            } => self.generate_function(name, params, return_type, body),
            Decl::ExternFunction { .. } => {}
        }
    }

    // ── Function generation ──

    fn generate_function(
        &mut self,
        name: &str,
        params: &[Param],
        _return_type: &Option<Type>,
        body: &[Stmt],
    ) {
        self.locals.clear();
        self.break_label.clear();
        self.continue_label.clear();
        self.loop_depth = 0;
        self.reset_temps();

        // ── Pass 1: allocate locals ──
        // Frame layout (low to high):
        //   [sp+0]   = saved fp
        //   [sp+8]   = saved ra
        //   [sp+16]  = first local (or param)
        //   ...
        let mut local_offset: i64 = 16;

        // Params first (they were passed in registers but stored on stack)
        for param in params.iter() {
            self.alloc_local_at(&param.name, &param.ty, &mut local_offset);
        }

        // Collect body locals
        self.collect_locals_stmts(body, &mut local_offset);

        // Reserve spill area (8 slots * 8 bytes = 64 bytes, aligned)
        local_offset = (local_offset + 7) & !7;
        self.spill_base = local_offset;
        self.spill_next = 0;
        local_offset += 64; // 8 spill slots

        let frame_size = ((local_offset + 15) & !15) as usize;
        self.current_frame_size = frame_size;

        // ── Emit prologue ──
        // Frame layout (sp at bottom):
        //   sp+0          : saved fp
        //   sp+8          : saved ra
        //   sp+16..       : locals
        //   sp+frame_size : top of frame
        self.emit(&format!("    .globl {}", name));
        self.emit(&format!("{}:", name));
        self.emit(&format!("    addi sp, sp, -{}", frame_size));
        self.emit("    sd fp, 0(sp)");
        self.emit("    sd ra, 8(sp)");
        self.emit("    addi fp, sp, 0");

        // Store parameters from arg regs to stack
        let param_regs = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];
        for (i, param) in params.iter().enumerate() {
            if i < 8 {
                if let Some(&(off, _)) = self.locals.get(&param.name) {
                    self.emit(&format!(
                        "    s{} {}, {}(fp)",
                        param.ty.mem_suffix(),
                        param_regs[i],
                        off
                    ));
                }
            }
        }

        // ── Pass 2: generate body ──
        for stmt in body {
            self.generate_stmt(stmt);
        }

        // Epilogue (only reached if function falls through without return)
        self.emit_epilogue(frame_size);
    }

    fn emit_epilogue(&mut self, frame_size: usize) {
        self.emit("    ld fp, 0(sp)");
        self.emit("    ld ra, 8(sp)");
        self.emit(&format!("    addi sp, sp, {}", frame_size));
        self.emit("    ret");
    }

    fn get_local(&self, name: &str) -> Option<&(i64, Type)> {
        self.locals.get(name)
    }

    // ── Local allocation (pass 1 helpers) ──

    fn alloc_local_at(&mut self, name: &str, ty: &Type, offset: &mut i64) -> i64 {
        let size = ty.size() as i64;
        let align = ty.align() as i64;
        // Align upward
        *offset = (*offset + align - 1) & !(align - 1);
        let off = *offset;
        *offset += size;
        self.locals.insert(name.to_string(), (off, ty.clone()));
        off
    }

    fn collect_locals_stmts(&mut self, stmts: &[Stmt], offset: &mut i64) {
        for stmt in stmts {
            self.collect_locals_stmt(stmt, offset);
        }
    }

    fn collect_locals_stmt(&mut self, stmt: &Stmt, offset: &mut i64) {
        match stmt {
            Stmt::Let { name, ty, .. } => {
                self.alloc_local_at(name, ty, offset);
            }
            Stmt::Block(stmts) => self.collect_locals_stmts(stmts, offset),
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                self.collect_locals_stmt(then_body, offset);
                if let Some(s) = else_body {
                    self.collect_locals_stmt(s, offset);
                }
            }
            Stmt::While { body, .. } | Stmt::Loop { body } => {
                self.collect_locals_stmt(body, offset);
            }
            _ => {}
        }
    }

    // ── Statement code generation ──

    fn generate_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, ty, init } => self.generate_let(name, ty, init),
            Stmt::Expr(expr) => {
                self.gen_expr_stmt(expr);
                self.reset_temps();
            }
            Stmt::Return(expr) => self.generate_return(expr),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => self.generate_if(cond, then_body, else_body),
            Stmt::While { cond, body } => self.generate_while(cond, body),
            Stmt::Loop { body } => self.generate_loop(body),
            Stmt::Break => self.generate_break(),
            Stmt::Continue => self.generate_continue(),
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.generate_stmt(s);
                }
            }
        }
    }

    fn generate_let(&mut self, name: &str, ty: &Type, init: &Option<Box<Expr>>) {
        if let Some((off, _)) = self.get_local(name).cloned() {
            if let Some(expr) = init {
                let reg = self.gen_expr_to_reg(expr);
                self.emit(&format!(
                    "    s{} {}, {}(fp)",
                    ty.mem_suffix(),
                    reg,
                    off
                ));
            }
        }
    }

    fn generate_return(&mut self, expr: &Option<Box<Expr>>) {
        if let Some(e) = expr {
            let reg = self.gen_expr_to_reg(e);
            if reg != "a0" {
                self.emit(&format!("    mv a0, {}", reg));
            }
        }
        self.emit_epilogue(self.current_frame_size);
    }

    fn generate_if(
        &mut self,
        cond: &Expr,
        then_body: &Stmt,
        else_body: &Option<Box<Stmt>>,
    ) {
        let else_lbl = self.fresh_label();
        let end_lbl = self.fresh_label();

        let cond_reg = self.gen_expr_to_reg(cond);
        self.emit(&format!("    beqz {}, {}", cond_reg, else_lbl));

        self.generate_stmt(then_body);
        self.emit(&format!("    j {}", end_lbl));

        self.emit(&format!("{}:", else_lbl));
        if let Some(s) = else_body {
            self.generate_stmt(s);
        }

        self.emit(&format!("{}:", end_lbl));
    }

    fn generate_while(&mut self, cond: &Expr, body: &Stmt) {
        let start = self.fresh_label();
        let end = self.fresh_label();

        self.loop_depth += 1;
        self.break_label.push(end.clone());
        self.continue_label.push(start.clone());

        self.emit(&format!("{}:", start));
        let cr = self.gen_expr_to_reg(cond);
        self.emit(&format!("    beqz {}, {}", cr, end));
        self.generate_stmt(body);
        self.emit(&format!("    j {}", start));
        self.emit(&format!("{}:", end));

        self.break_label.pop();
        self.continue_label.pop();
        self.loop_depth -= 1;
    }

    fn generate_loop(&mut self, body: &Stmt) {
        let start = self.fresh_label();
        let end = self.fresh_label();

        self.loop_depth += 1;
        self.break_label.push(end.clone());
        self.continue_label.push(start.clone());

        self.emit(&format!("{}:", start));
        self.generate_stmt(body);
        self.emit(&format!("    j {}", start));
        self.emit(&format!("{}:", end));

        self.break_label.pop();
        self.continue_label.pop();
        self.loop_depth -= 1;
    }

    fn generate_break(&mut self) {
        if let Some(l) = self.break_label.last() {
            self.emit(&format!("    j {}", l));
        }
    }

    fn generate_continue(&mut self) {
        if let Some(l) = self.continue_label.last() {
            self.emit(&format!("    j {}", l));
        }
    }

    // ── Expression code generation ──

    fn gen_expr_to_reg(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLiteral(v) => self.gen_int_literal(*v),
            Expr::StringLiteral(s) => self.gen_string_literal(s),
            Expr::Ident(n) => self.gen_ident(n),
            Expr::Binary { op, left, right } => self.gen_binary(*op, left, right),
            Expr::Unary { op, operand } => self.gen_unary(*op, operand),
            Expr::Call { callee, args } => self.gen_call(callee, args),
            Expr::Assign { target, op, value } => {
                self.gen_assign(target, *op, value)
            }
        }
    }

    fn gen_expr_stmt(&mut self, expr: &Expr) {
        match expr {
            Expr::Call { callee, args } => {
                self.gen_call(callee, args);
            }
            Expr::Assign { target, op, value } => {
                self.gen_assign(target, *op, value);
            }
            _ => {
                self.gen_expr_to_reg(expr);
            }
        }
    }

    fn gen_int_literal(&mut self, val: i64) -> String {
        let r = self.alloc_temp().to_string();
        if val >= -2048 && val < 2048 {
            self.emit(&format!("    addi {}, zero, {}", r, val));
        } else {
            self.emit(&format!("    li {}, {}", r, val));
        }
        r
    }

    fn gen_string_literal(&mut self, s: &str) -> String {
        let lbl = self.fresh_string_label();
        self.strings.push((lbl.clone(), s.to_string()));
        let r = self.alloc_temp().to_string();
        self.emit(&format!("    lla {}, {}", r, lbl));
        r
    }

    fn gen_ident(&mut self, name: &str) -> String {
        if let Some((off, ty)) = self.get_local(name).cloned() {
            let r = self.alloc_temp().to_string();
            let lsuf = ty.load_suffix(ty.is_signed());
            self.emit(&format!("    l{} {}, {}(fp)", lsuf, r, off));
            r
        } else {
            // Function address
            let r = self.alloc_temp().to_string();
            self.emit(&format!("    lla {}, {}", r, name));
            r
        }
    }

    fn gen_binary(&mut self, op: BinaryOp, left: &Expr, right: &Expr) -> String {
        let lr = self.gen_expr_to_reg(left);
        // Spill left operand to stack: the right side may contain function
        // calls that clobber caller-saved registers (t0-t6).
        let spill_slot = self.spill_reg(&lr);
        let rr = self.gen_expr_to_reg(right);
        let lr2 = self.unspill_reg(spill_slot);
        let dr = self.alloc_temp().to_string();

        match op {
            BinaryOp::Add => self.emit(&format!("    add {}, {}, {}", dr, lr2, rr)),
            BinaryOp::Sub => self.emit(&format!("    sub {}, {}, {}", dr, lr2, rr)),
            BinaryOp::Mul => self.emit(&format!("    mul {}, {}, {}", dr, lr2, rr)),
            BinaryOp::Div => self.emit(&format!("    div {}, {}, {}", dr, lr2, rr)),
            BinaryOp::Rem => self.emit(&format!("    rem {}, {}, {}", dr, lr2, rr)),
            BinaryOp::Lt => self.emit(&format!("    slt {}, {}, {}", dr, lr2, rr)),
            BinaryOp::Gt => self.emit(&format!("    slt {}, {}, {}", dr, rr, lr2)),
            BinaryOp::LtEq => {
                let t = self.alloc_temp().to_string();
                self.emit(&format!("    slt {}, {}, {}", t, rr, lr2));
                self.emit(&format!("    xori {}, {}, 1", dr, t));
            }
            BinaryOp::GtEq => {
                let t = self.alloc_temp().to_string();
                self.emit(&format!("    slt {}, {}, {}", t, lr2, rr));
                self.emit(&format!("    xori {}, {}, 1", dr, t));
            }
            BinaryOp::Eq => {
                let t = self.alloc_temp().to_string();
                self.emit(&format!("    sub {}, {}, {}", t, lr2, rr));
                self.emit(&format!("    sltiu {}, {}, 1", dr, t));
            }
            BinaryOp::NotEq => {
                let t = self.alloc_temp().to_string();
                self.emit(&format!("    sub {}, {}, {}", t, lr2, rr));
                self.emit(&format!("    sltu {}, zero, {}", dr, t));
            }
            BinaryOp::And | BinaryOp::Or => {
                self.emit(&format!("    li {}, 0", dr));
            }
        }
        dr
    }

    fn gen_unary(&mut self, op: UnaryOp, operand: &Expr) -> String {
        let or = self.gen_expr_to_reg(operand);
        let dr = self.alloc_temp().to_string();

        match op {
            UnaryOp::Neg => self.emit(&format!("    neg {}, {}", dr, or)),
            UnaryOp::Not => self.emit(&format!("    seqz {}, {}", dr, or)),
            UnaryOp::Addr => {
                if let Expr::Ident(name) = operand {
                    if let Some((off, _)) = self.get_local(name) {
                        self.emit(&format!("    addi {}, fp, {}", dr, off));
                    } else {
                        self.emit(&format!("    mv {}, {}", dr, or));
                    }
                } else {
                    self.emit(&format!("    mv {}, {}", dr, or));
                }
            }
            UnaryOp::Deref => {
                self.emit(&format!("    ld {}, 0({})", dr, or));
            }
        }
        dr
    }

    fn gen_call(&mut self, callee: &Expr, args: &[Expr]) -> String {
        self.reset_temps();

        let arg_regs = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

        let mut temps: Vec<String> = Vec::new();
        for arg in args {
            temps.push(self.gen_expr_to_reg(arg));
        }

        for (i, t) in temps.iter().enumerate() {
            if i < 8 && t != arg_regs[i] {
                self.emit(&format!("    mv {}, {}", arg_regs[i], t));
            }
        }

        let callee_name = match callee {
            Expr::Ident(n) => n.clone(),
            _ => {
                let r = self.gen_expr_to_reg(callee);
                r
            }
        };

        self.emit(&format!("    call {}", callee_name));
        "a0".to_string()
    }

    fn gen_assign(
        &mut self,
        target: &Expr,
        op: AssignOp,
        value: &Expr,
    ) -> String {
        match target {
            Expr::Ident(name) => {
                if let Some((off, ty)) = self.get_local(name).cloned() {
                    match op {
                        AssignOp::Simple => {
                            let vr = self.gen_expr_to_reg(value);
                            self.emit(&format!(
                                "    s{} {}, {}(fp)",
                                ty.mem_suffix(),
                                vr,
                                off
                            ));
                            vr
                        }
                        _ => {
                            let tr = self.alloc_temp().to_string();
                            let lsuf = ty.load_suffix(ty.is_signed());
                            self.emit(&format!(
                                "    l{} {}, {}(fp)",
                                lsuf, tr, off
                            ));
                            // Spill current value before evaluating right side
                            let spill_slot = self.spill_reg(&tr);
                            let vr = self.gen_expr_to_reg(value);
                            let tr2 = self.unspill_reg(spill_slot);
                            match op {
                                AssignOp::Add => self
                                    .emit(&format!("    add {}, {}, {}", tr2, tr2, vr)),
                                AssignOp::Sub => self
                                    .emit(&format!("    sub {}, {}, {}", tr2, tr2, vr)),
                                AssignOp::Mul => self
                                    .emit(&format!("    mul {}, {}, {}", tr2, tr2, vr)),
                                AssignOp::Div => self
                                    .emit(&format!("    div {}, {}, {}", tr2, tr2, vr)),
                                AssignOp::Rem => self
                                    .emit(&format!("    rem {}, {}, {}", tr2, tr2, vr)),
                                _ => {}
                            }
                            self.emit(&format!(
                                "    s{} {}, {}(fp)",
                                ty.mem_suffix(),
                                tr2,
                                off
                            ));
                            tr2
                        }
                    }
                } else {
                    let vr = self.gen_expr_to_reg(value);
                    eprintln!("Warning: assign to unknown var '{}'", name);
                    vr
                }
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                operand,
            } => {
                let ar = self.gen_expr_to_reg(operand);
                let vr = self.gen_expr_to_reg(value);
                self.emit(&format!("    sd {}, 0({})", vr, ar));
                vr
            }
            _ => {
                let vr = self.gen_expr_to_reg(value);
                eprintln!("Warning: assign to unsupported target");
                vr
            }
        }
    }
}
