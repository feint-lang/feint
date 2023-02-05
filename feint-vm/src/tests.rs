use feint_builtins::types::{
    code::{Code, Inst},
    Module,
};
use feint_builtins::BUILTINS;
use feint_util::op::BinaryOperator;

use crate::*;

#[test]
fn execute_simple_program() {
    let mut code = Code::with_chunk(vec![
        Inst::LoadConst(0),
        Inst::LoadConst(1),
        Inst::BinaryOp(BinaryOperator::Add),
    ]);
    code.add_const(BUILTINS.int(1));
    code.add_const(BUILTINS.int(2));
    let module = Module::new(
        BUILTINS.module_type(),
        "test".to_owned(),
        "test".to_owned(),
        code,
        None,
    );
    let mut vm = VM::default();
    assert!(matches!(vm.execute_module(&module, 0), Ok(())));
    assert!(matches!(vm.state, VMState::Idle(Some(_))));
}
