use crate::types::{ObjectExt, ObjectRef};
use crate::vm::RuntimeContext;

#[test]
fn test_float() {
    let ctx = RuntimeContext::default();

    let float1 = ctx.builtins.new_float(0.0);
    let float2 = ctx.builtins.new_float(0.0);
    let float3 = ctx.builtins.new_float(1.0);

    assert!(float1.class().is(&float2.class()));
    assert!(float2.class().is(&float3.class()));

    assert!(float1.is(&*float1));
    assert!(!float1.is(&*float2));
    assert!(!float1.is(&*float3));

    assert!(float1.is_equal(&*float2, &ctx));
    assert!(!float1.is_equal(&*float3, &ctx));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());
}

#[test]
fn test_compare_float_to_int() {
    let ctx = RuntimeContext::default();
    let float = ctx.builtins.new_float(1.0);
    let int = ctx.builtins.new_int(1u8);
    assert!(float.is_equal(&*int, &ctx));
    assert!(int.is_equal(&*float, &ctx));
}

#[test]
fn test_custom() {
    let ctx = RuntimeContext::default();

    let t1 = ctx.builtins.new_type("test", "Custom1");
    let t1_obj1 = ctx.builtins.new_custom_instance(t1.clone());
    let t1_obj2 = ctx.builtins.new_custom_instance(t1.clone());
    let t1_obj3 = ctx.builtins.new_custom_instance(t1.clone());

    assert!((t1.clone() as ObjectRef).get_attr("$id", &ctx, t1.clone()).is_ok());
    assert!((t1.clone() as ObjectRef).get_attr("$type", &ctx, t1.clone()).is_ok());
    assert!(t1_obj1.get_attr("$id", &ctx, t1_obj1.clone()).is_ok());
    assert!(t1_obj1.get_attr("$type", &ctx, t1_obj1.clone()).is_ok());

    t1_obj3
        .set_attr("value", ctx.builtins.new_int(1), &ctx)
        .expect("Could not set attribute");

    let t2 = ctx.builtins.new_type("test", "Custom2");
    let t2_obj1 = ctx.builtins.new_custom_instance(t2.clone());

    // An object should be equal to itself.
    assert!(t1_obj1.is_equal(&*t1_obj1, &ctx));

    // An object should be equal to an object of the SAME type with
    // the same attributes.
    assert!(t1_obj1.is_equal(&*t1_obj2, &ctx));

    // An object should NOT be equal to an object of the SAME type with
    // the DIFFERENT attributes.
    assert!(!t1_obj1.is_equal(&*t1_obj3, &ctx));

    // An object should NOT be equal to an object of a DIFFERENT type,
    // regardless of attributes.
    assert!(!t1_obj1.is_equal(&*t2_obj1, &ctx));
}
