use crate::types::new;
use crate::util::BinaryOperator;
use crate::vm::*;

#[test]
fn execute_simple_program() {
    let mut vm = VM::default();
    let mut code = Code::new();
    let i = code.add_const(new::int(1));
    let j = code.add_const(new::int(2));
    code.push_inst(Inst::LoadConst(i));
    code.push_inst(Inst::LoadConst(j));
    code.push_inst(Inst::BinaryOp(BinaryOperator::Add));
    assert!(matches!(vm.execute_code(None, &code, 0), Ok(())));
    assert!(matches!(vm.state, VMState::Idle(Some(_))));
}
