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

    assert_eq!(float1.class().id(), float2.class().id());
    assert_eq!(float2.class().id(), float3.class().id());
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
//
// #[test]
// fn test_custom() {
//     let type_1 = Type::new("test", "Custom1");
//     let value_1 = types::make_int(BigInt::from(0));
//     let mut attributes_1 = HashMap::new();
//     attributes_1.insert("value".to_owned(), value_1);
//     let obj_1 = type_1.instance(attributes_1);
//
//     let type_2 = Type::new("test", "Custom2");
//     let value_2 = types::make_int(BigInt::from(0));
//     let mut attributes_2 = HashMap::new();
//     attributes_2.insert("value".to_owned(), value_2);
//     let obj_2 = type_2.instance(attributes_2);
//
//     assert_ne!(type_1, type_2);
//     assert_eq!(obj_1, obj_2);
//
//     if let Ok(value) = obj_1.get_attribute("value") {
//         assert!(true);
//     } else {
//         assert!(false);
//     }
// }
