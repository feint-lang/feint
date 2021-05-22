use std::rc::Rc;
use std::sync::Arc;

use super::builtins;
use super::class::Type;
use super::complex::ComplexObject;
use super::object::Object;

#[test]
fn test_float() {
    let float1 = builtins::Float::from(0.0);
    let float2 = builtins::Float::from(0.0);
    let float3 = builtins::Float::from(1.0);

    assert!(float1.class().is(&float2.class()));
    assert!(float2.class().is(&float3.class()));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());

    // compare concrete types
    assert_eq!(float1, float2);
    assert_ne!(float1, float3);

    // compare via trait
    assert!(Object::eq(&float1, &float2));
    assert!(!Object::eq(&float1, &float3));
}

#[test]
fn test_compare_float_to_int() {
    let float = builtins::Float::from(1.0);
    let int = builtins::Int::from(1);
    assert!(Object::eq(&int, &float)); // compare via trait
    assert!(Object::eq(&float, &int)); // compare via trait
}

#[test]
fn test_custom() {
    let type_1 = Arc::new(Type::new("test", "Custom1"));
    let mut obj_1 = ComplexObject::new(type_1.clone());
    let value_1 = Rc::new(builtins::Int::from(1));
    obj_1.set_attribute("value", value_1);

    let type_2 = Arc::new(Type::new("test", "Custom2"));
    let mut obj_2 = ComplexObject::new(type_2.clone());
    let value_2 = Rc::new(builtins::Int::from(1));
    obj_2.set_attribute("value", value_2);

    assert_eq!(obj_1, obj_2); // compare concrete types
    assert!(Object::eq(&obj_1, &obj_2)); // compare via trait
}
