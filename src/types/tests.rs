use std::rc::Rc;

use crate::vm::RuntimeContext;

use super::class::Type;
use super::complex::ComplexObject;
use super::object::{Object, ObjectRef};

#[test]
fn test_float() {
    let ctx = RuntimeContext::default();

    let float1 = ctx.builtins.new_float(0.0);
    let float2 = ctx.builtins.new_float(0.0);
    let float3 = ctx.builtins.new_float(1.0);

    assert!(float1.class().is(&float2.class()));
    assert!(float2.class().is(&float3.class()));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());

    assert!(float1.is_equal(&float2, &ctx).unwrap());
    assert!(!float1.is_equal(&float3, &ctx).unwrap());
}

#[test]
fn test_compare_float_to_int() {
    let ctx = RuntimeContext::default();
    let float = ctx.builtins.new_float(1.0);
    let int = ctx.builtins.new_int(1u8);
    assert!(float.is_equal(&int, &ctx).unwrap());
    assert!(int.is_equal(&float, &ctx).unwrap());
}

#[test]
fn test_custom() {
    let ctx = RuntimeContext::default();

    let t1 = Rc::new(Type::new("test", "Custom1"));
    let t1_obj1 = Rc::new(ComplexObject::new(t1.clone()));
    let t1_obj2 = Rc::new(ComplexObject::new(t1.clone()));
    let t1_obj3 = Rc::new(ComplexObject::new(t1.clone()));
    t1_obj3
        .set_attribute("value", ctx.builtins.new_int(1))
        .expect("Could not set attribute");

    let t2 = Rc::new(Type::new("test", "Custom2"));
    let t2_obj1 = Rc::new(ComplexObject::new(t2.clone()));

    // XXX: All the cloning and casting below seems wonky.

    // An object should be equal to itself.
    match t1_obj1.is_equal(&(t1_obj1.clone() as ObjectRef), &ctx) {
        Ok(result) => assert!(result),
        Err(_) => assert!(false, "Could not compare custom objects"),
    }

    // An object should be equal to an object of the SAME type with
    // the same attributes.
    match t1_obj1.is_equal(&(t1_obj2.clone() as ObjectRef), &ctx) {
        Ok(result) => assert!(result),
        Err(_) => assert!(false, "Could not compare custom objects"),
    }

    // An object should NOT be equal to an object of the SAME type with
    // the DIFFERENT attributes.
    match t1_obj1.is_equal(&(t1_obj3.clone() as ObjectRef), &ctx) {
        Ok(result) => assert!(!result),
        Err(_) => assert!(false, "Could not compare custom objects"),
    }

    // An object should NOT equal to an object of a DIFFERENT type,
    // regardless of attributes.
    match t1_obj1.is_equal(&(t2_obj1.clone() as ObjectRef), &ctx) {
        Ok(result) => assert!(!result),
        Err(_) => assert!(false, "Could not compare custom objects"),
    }
}
