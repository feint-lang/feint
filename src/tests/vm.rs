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
    code.push_inst(Inst::Halt(0));
    assert!(matches!(vm.execute_code(None, &code, 0), Ok(VMState::Halted(0))));
}

#[test]
fn test_add_get_global_const() {
    let mut ctx = RuntimeContext::new();
    let int = new::int(0);
    let int_copy = int.clone();
    let index = ctx.add_global_const(int.clone());
    let retrieved = ctx.get_global_const(index).unwrap();
    let retrieved = retrieved.read().unwrap();
    assert!(retrieved.class().read().unwrap().is(&*int_copy
        .read()
        .unwrap()
        .class()
        .read()
        .unwrap()));
    assert!(retrieved.is(&*int_copy.read().unwrap()));
    assert!(retrieved.is_equal(&*int_copy.read().unwrap()));
    assert_eq!(retrieved.id(), int_copy.read().unwrap().id());
}
