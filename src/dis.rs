use crate::vm::{Code, Inst};
use std::fmt;

pub fn dis(code: &Code) {
    for (ip, inst) in code.iter_chunk().enumerate() {
        println!("{ip:0>4} {}", format_inst(code, inst));
    }
    for obj_ref in code.iter_constants() {
        let obj = obj_ref.read().unwrap();
        if let Some(func) = obj.down_to_func() {
            println!();
            let heading = format!("{func:?} ");
            println!("{:=<79}", heading);
            dis(&func.code);
        }
    }
}

fn align<T: fmt::Display>(name: &str, value: T) -> String {
    format!("{name: <w$}{value:}", w = 32)
}

fn format_inst(code: &Code, inst: &Inst) -> String {
    use Inst::*;
    match inst {
        NoOp => align("NOOP", "ø"),
        Pop => align("POP", ""),
        PopN(n) => align("POP_N", n),
        LoadGlobalConst(index) => {
            let index = *index;
            let op_code = "LOAD_GLOBAL_CONST";
            if (3..=259).contains(&index) {
                align(op_code, (index - 3).to_string())
            } else {
                align(op_code, "[unknown]")
            }
        }
        LoadNil => align("LOAD_NIL", "nil"),
        LoadTrue => align("LOAD_TRUE", "true"),
        LoadFalse => align("LOAD_FALSE", "false"),
        LoadConst(index) => {
            let constant = match code.get_const(*index) {
                Ok(obj) => obj.read().unwrap().to_string(),
                Err(err) => err.to_string(),
            };
            align("LOAD_CONST", format!("{index} ({constant})"))
        }
        ScopeStart => align("SCOPE_START", "->"),
        ScopeEnd => align("SCOPE_END", ""),
        StoreLocal(index) => align("STORE_LOCAL", index),
        LoadLocal(index) => align("LOAD_LOCAL", index),
        DeclareVar(name) => align("DECLARE_VAR", name),
        AssignVar(name) => align("ASSIGN_VAR", name),
        LoadVar(name) => align("LOAD_VAR", name),
        Jump(addr, _) => align("JUMP", format!("{addr}",)),
        JumpPushNil(addr, _) => align("JUMP_PUSH_NIL", format!("{addr}",)),
        JumpIf(addr, _) => align("JUMP_IF", format!("{addr}",)),
        JumpIfNot(addr, _) => align("JUMP_IF_NOT", format!("{addr}",)),
        JumpIfElse(if_addr, else_addr, _) => {
            align("JUMP_IF_ELSE", format!("{if_addr} : {else_addr}"))
        }
        UnaryOp(op) => align("UNARY_OP", op),
        UnaryCompareOp(op) => align("UNARY_COMPARE_OP", op),
        BinaryOp(op) => align("BINARY_OP", op),
        CompareOp(op) => align("COMPARE_OP", op),
        InplaceOp(op) => align("INPLACE_OP", op),
        Call(num_args) => align("CALL", num_args),
        Return => align("RETURN", ""),
        MakeString(n) => align("MAKE_STRING", n),
        MakeTuple(n) => align("MAKE_TUPLE", n),
        Halt(code) => align("HALT", code),
        HaltTop => align("HALT_TOP", ""),
        // None of the following should ever appear in the list. If they
        // do, something has gone horribly wrong.
        Placeholder(addr, inst, message) => {
            let formatted_inst = format_inst(code, inst);
            align("PLACEHOLDER", format!("{formatted_inst} @ {addr} ({message})"))
        }
        BreakPlaceholder(addr, _) => align("PLACEHOLDER", format!("BREAK @ {addr}")),
        ContinuePlaceholder(addr, _) => {
            align("PLACEHOLDER", format!("CONTINUE @ {addr}"))
        }
    }
}
