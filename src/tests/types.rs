use crate::types::{create, Namespace, ObjectTrait, ObjectTraitExt, TypeTraitExt};

#[test]
fn test_float() {
    let float1 = create::new_float(0.0);
    let float2 = create::new_float(0.0);
    let float3 = create::new_float(1.0);

    // TODO:
    assert!(float1.class().is(&*float2.class()));
    assert!(float2.class().is(&*float3.class()));

    assert!(float1.is(&*float1));
    assert!(!float1.is(&*float2));
    assert!(!float1.is(&*float3));

    assert!(float1.is_equal(&*float2));
    assert!(!float1.is_equal(&*float3));

    assert_ne!(float1.id(), float2.id());
    assert_ne!(float2.id(), float3.id());
}

#[test]
fn test_compare_float_to_int() {
    let float = create::new_float(1.0);
    let int = create::new_int(1);
    assert!(float.is_equal(&*int));
    assert!(int.is_equal(&*float));
}

#[test]
fn test_custom() {
    let mod1 = create::new_module("test1", Namespace::new());

    let t1 = create::new_custom_type(mod1, "Custom1");

    let mut ns = Namespace::new();
    ns.add_obj("value", create::new_nil());
    let t1_obj1 = create::new_custom_instance(t1.clone(), ns);

    let mut ns = Namespace::new();
    ns.add_obj("value", create::new_nil());
    let t1_obj2 = create::new_custom_instance(t1.clone(), ns);

    let mut ns = Namespace::new();
    ns.add_obj("value", create::new_nil());
    let t1_obj3 = create::new_custom_instance(t1.clone(), ns);

    assert!(t1.clone().get_attr("$id").is_ok());
    assert!(t1.clone().get_attr("$type").is_ok());
    assert!(t1_obj1.get_attr("$id").is_ok());
    assert!(t1_obj1.get_attr("$type").is_ok());

    let was_set = t1_obj3.namespace().borrow_mut().set_obj("value", create::new_int(1));
    assert!(was_set, "Could not set `value` on t1_obj3");
    assert!(t1_obj3.get_attr("value").is_ok());
    assert!(t1_obj3.get_attr("value").unwrap().is_equal(&*create::new_int(1)));

    let mod2 = create::new_module("test2", Namespace::new());

    let t2 = create::new_custom_type(mod2, "Custom2");
    let t2_obj1 = create::new_custom_instance(t2, Namespace::new());

    // An object should be equal to itself.
    assert!(t1_obj1.is_equal(&*t1_obj1));

    // An object should be equal to an object of the SAME type with
    // the same attributes.
    assert!(t1_obj1.is_equal(&*t1_obj2));

    // An object should NOT be equal to an object of the SAME type with
    // the DIFFERENT attributes.
    assert!(!t1_obj1.is_equal(&*t1_obj3));

    // An object should NOT be equal to an object of a DIFFERENT type,
    // regardless of attributes.
    assert!(!t1_obj1.is_equal(&*t2_obj1));
}
