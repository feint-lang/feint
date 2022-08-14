use std::fmt;

use crate::vm::{Code, Inst};

pub struct Disassembler {
    curr_line_no: usize,
    new_line: bool,
}

impl Disassembler {
    pub fn new() -> Self {
        Self { curr_line_no: 0, new_line: false }
    }

    pub fn disassemble(&mut self, code: &Code) {
        use Inst::*;
        let width = 8;
        let iter = code.iter_chunk().enumerate();
        println!("{: <width$}    {:<width$}    INSTRUCTION", "LINE", "IP");
        for (ip, inst) in iter {
            let line = self.format_inst(code, inst);
            let line_no = if matches!(inst, Halt(_) | Pop) {
                println!();
                "".to_string()
            } else if self.new_line {
                println!();
                self.new_line = false;
                self.curr_line_no.to_string()
            } else {
                "".to_string()
            };
            println!("{line_no: <width$}    {ip:0>width$}    {line}");
        }
        for obj_ref in code.iter_constants() {
            let obj = obj_ref.read().unwrap();
            if let Some(func) = obj.down_to_func() {
                println!();
                let heading = format!("{func:?} ");
                println!("{:=<79}", heading);
                self.disassemble(&func.code);
            }
        }
    }

    /// Align instruction name and any additional data, such as a
    /// constant index, var name, etc.
    fn align<T: fmt::Display>(&self, name: &str, value: T) -> String {
        format!("{name: <w$}{value:}", w = 24)
    }

    fn format_inst(&mut self, code: &Code, inst: &Inst) -> String {
        use Inst::*;
        match inst {
            NoOp => self.align("NOOP", "ø"),
            Pop => self.align("POP", ""),
            LoadGlobalConst(index) => {
                let index = *index;
                let op_code = "LOAD_GLOBAL_CONST";
                if (3..=259).contains(&index) {
                    self.align(
                        op_code,
                        format!("{index} ({})", (index - 3).to_string()),
                    )
                } else {
                    self.align(op_code, format!("{index} ([unknown])"))
                }
            }
            LoadNil => self.align("LOAD_NIL", "nil"),
            LoadTrue => self.align("LOAD_TRUE", "true"),
            LoadFalse => self.align("LOAD_FALSE", "false"),
            ScopeStart => self.align("SCOPE_START", "->"),
            ScopeEnd => self.align("SCOPE_END", ""),
            StatementStart(start, _) => {
                self.new_line = start.line != self.curr_line_no;
                self.curr_line_no = start.line;
                self.align("STATEMENT_START", "")
            }
            LoadConst(index) => {
                let constant = match code.get_const(*index) {
                    Ok(obj) => obj.read().unwrap().to_string(),
                    Err(err) => err.to_string(),
                };
                self.align("LOAD_CONST", format!("{index} ({constant})"))
            }
            StoreLocal(index, captured) => {
                self.align("STORE_LOCAL", format!("{index} : captured = {captured}"))
            }
            LoadLocal(index) => self.align("LOAD_LOCAL", index),
            LoadCell(index) => self.align("LOAD_CELL", index),
            DeclareVar(name) => self.align("DECLARE_VAR", name),
            AssignVar(name) => self.align("ASSIGN_VAR", name),
            LoadVar(name) => self.align("LOAD_VAR", name),
            Jump(addr, _) => self.align("JUMP", format!("{addr}",)),
            JumpPushNil(addr, _) => self.align("JUMP_PUSH_NIL", format!("{addr}",)),
            JumpIf(addr, _) => self.align("JUMP_IF", format!("{addr}",)),
            JumpIfNot(addr, _) => self.align("JUMP_IF_NOT", format!("{addr}",)),
            JumpIfElse(if_addr, else_addr, _) => {
                self.align("JUMP_IF_ELSE", format!("{if_addr} : {else_addr}"))
            }
            UnaryOp(op) => self.align("UNARY_OP", op),
            UnaryCompareOp(op) => self.align("UNARY_COMPARE_OP", op),
            BinaryOp(op) => self.align("BINARY_OP", op),
            CompareOp(op) => self.align("COMPARE_OP", op),
            InplaceOp(op) => self.align("INPLACE_OP", op),
            Call(num_args) => self.align("CALL", num_args),
            Return => self.align("RETURN", ""),
            MakeString(n) => self.align("MAKE_STRING", n),
            MakeTuple(n) => self.align("MAKE_TUPLE", n),
            MakeList(n) => self.align("MAKE_LIST", n),
            MakeMap(n) => self.align("MAKE_MAP", n),
            MakeClosure(index, count) => {
                let func = match code.get_const(*index) {
                    Ok(obj) => obj.read().unwrap().to_string(),
                    Err(err) => err.to_string(),
                };
                self.align("MAKE_CLOSURE", format!("{func} ({count} captured)"))
            }
            LoadModule(name) => self.align("IMPORT", name),
            Halt(code) => self.align("HALT", code),
            HaltTop => self.align("HALT_TOP", ""),
            // None of the following should ever appear in the list. If they
            // do, something has gone horribly wrong.
            Placeholder(addr, inst, message) => {
                let formatted_inst = self.format_inst(code, inst);
                self.align(
                    "PLACEHOLDER",
                    format!("{formatted_inst} @ {addr} ({message})"),
                )
            }
            VarPlaceholder(addr, name) => {
                self.align("PLACEHOLDER", format!("VAR {name} @ {addr}"))
            }
            BreakPlaceholder(addr, _) => {
                self.align("PLACEHOLDER", format!("BREAK @ {addr}"))
            }
            ContinuePlaceholder(addr, _) => {
                self.align("PLACEHOLDER", format!("CONTINUE @ {addr}"))
            }
            ReturnPlaceholder(addr, _) => {
                self.align("PLACEHOLDER", format!("RETURN @ {addr}"))
            }
        }
    }
}
