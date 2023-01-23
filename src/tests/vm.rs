use crate::types::{new, Module, Namespace};
use crate::util::BinaryOperator;
use crate::vm::*;

#[test]
fn execute_simple_program() {
    let mut code = Code::with_chunk(vec![
        Inst::LoadConst(0),
        Inst::LoadConst(1),
        Inst::BinaryOp(BinaryOperator::Add),
    ]);
    code.add_const(new::int(1));
    code.add_const(new::int(2));
    let module =
        Module::new("test".to_owned(), "test".to_owned(), Namespace::new(), code, None);
    let mut vm = VM::default();
    assert!(matches!(vm.execute_module(&module, 0), Ok(())));
    assert!(matches!(vm.state, VMState::Idle(Some(_))));
}
