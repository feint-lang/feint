use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use super::*;

#[test]
fn test_float() {
    let builtins = builtins::Builtins::new();
    let float1 = builtins.new_float(0.0);
    let float2 = builtins.new_float(0.0);
    let float3 = builtins.new_float(1.0);

    assert!(float1.class().is(float2.class()));
    assert!(float2.class().is(float3.class()));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());

    assert_eq!(float1, float2);
    assert_ne!(float1, float3);
}

#[test]
fn test_compare_float_to_int() {
    let builtins = builtins::Builtins::new();
    let float = builtins.new_float(1.0);
    let int = builtins.new_int(BigInt::from(1));
    assert_eq!(float, int);
    println!("float={} int={}", float, int);
}

#[test]
fn test_custom() {
    let builtins = builtins::Builtins::new();

    let type_1 = Rc::new(Type::new("test", "Custom1", None));
    let mut obj_1 = Object::new(type_1.clone());
    let value_1 = Rc::new(builtins.new_int(BigInt::from(1)));
    obj_1.set_attribute("value", value_1);
}
