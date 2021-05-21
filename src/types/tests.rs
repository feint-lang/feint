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

    assert!(float1.class().is(&float2.class()));
    assert!(float2.class().is(&float3.class()));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());

    assert_eq!(&float1, &float2);
    assert!(float1.eq(&float2));

    assert_ne!(&float1, &float3);
    assert!(float1.ne(&float3));
}

#[test]
fn test_compare_float_to_int() {
    let builtins = builtins::Builtins::new();
    let float = builtins.new_float(1.0);
    let int = builtins.new_int(BigInt::from(1));
    assert_eq!(float.as_ref(), int.as_ref());
}

#[test]
fn test_custom() {
    let builtins = builtins::Builtins::new();

    let type_1 = Rc::new(Type::new("test", "Custom1"));
    let mut obj_1 = ComplexObject::new(type_1.clone());
    let value_1 = builtins.new_int(BigInt::from(1));
    obj_1.set_attribute("value", value_1);

    let type_2 = Rc::new(Type::new("test", "Custom2"));
    let mut obj_2 = ComplexObject::new(type_2.clone());
    let value_2 = builtins.new_int(BigInt::from(1));
    obj_2.set_attribute("value", value_2);

    assert_eq!(obj_1, obj_2);
}
