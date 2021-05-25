use std::cell::RefCell;
use std::rc::Rc;

use super::builtins::Builtins;
use super::class::Type;
use super::complex::ComplexObject;
use super::object::Object;

#[test]
fn test_float() {
    let builtins = Builtins::new();

    let float1 = builtins.new_float(0.0);
    let float2 = builtins.new_float(0.0);
    let float3 = builtins.new_float(1.0);

    assert!(float1.class().is(&float2.class()));
    assert!(float2.class().is(&float3.class()));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());

    assert!(float1.is_equal(float2));
    assert!(!float1.is_equal(float3));
}

#[test]
fn test_compare_float_to_int() {
    let builtins = Builtins::new();

    let float = builtins.new_float(1.0);
    let int = builtins.new_int(1u8);

    assert!(float.is_equal(int.clone()));
    assert!(int.is_equal(float.clone()));
}

#[test]
fn test_custom() {
    let builtins = Builtins::new();

    let type_1 = Rc::new(Type::new("test", "Custom1"));
    let mut obj_1 = ComplexObject::new(type_1);
    let value_1 = builtins.new_int(1);
    obj_1.set_attribute("value", value_1);

    let type_2 = Rc::new(Type::new("test", "Custom2"));
    let mut obj_2 = ComplexObject::new(type_2);
    let value_2 = builtins.new_int(1);
    obj_2.set_attribute("value", value_2);

    // FIXME: ???
    assert!(obj_1.is_equal(Rc::new(obj_2)))
}
