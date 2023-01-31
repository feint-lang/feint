use feint_builtins::types::code::{Code, Inst};
use feint_builtins::types::{new, Module};
use feint_util::op::BinaryOperator;

use crate::*;

#[test]
fn execute_simple_program() {
    let mut code = Code::with_chunk(vec![
        Inst::LoadConst(0),
        Inst::LoadConst(1),
        Inst::BinaryOp(BinaryOperator::Add),
    ]);
    code.add_const(new::int(1));
    code.add_const(new::int(2));
    let module = Module::new("test".to_owned(), "test".to_owned(), code, None);
    let mut vm = VM::default();
    assert!(matches!(vm.execute_module(&module, 0), Ok(())));
    assert!(matches!(vm.state, VMState::Idle(Some(_))));
}