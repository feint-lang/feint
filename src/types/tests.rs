use num_bigint::BigInt;

use super::*;

#[test]
fn test_float() {
    let float1 = types::make_float(0.0);
    let float2 = types::make_float(0.0);
    let float3 = types::make_float(1.0);

    assert_eq!(float1.class.id(), float2.class.id());
    assert_eq!(float2.class.id(), float3.class.id());
    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());
    assert!(float1.is_equal(&float2));
    assert!(!float1.is_equal(&float3));
}

#[test]
fn test_compare_float_to_int() {
    let float = types::make_float(0.0);
    let int = types::make_int(BigInt::from(0));
    assert_eq!(float, int);
}

#[test]
fn test_custom() {
    let type_1 = Type::new("test", "Custom1", vec!["value"]);
    let value_1 = AttributeValue::Object(types::make_int(BigInt::from(0)));
    let obj_1 = type_1.instance(vec![value_1]);

    let type_2 = Type::new("test", "Custom2", vec!["value"]);
    let value_2 = AttributeValue::Object(types::make_int(BigInt::from(0)));
    let obj_2 = type_2.instance(vec![value_2]);

    assert_ne!(type_1, type_2);
    assert_eq!(obj_1, obj_2);

    if let Ok(value) = obj_1.get_attribute("value") {
        assert!(true);
    } else {
        assert!(false);
    }
}
