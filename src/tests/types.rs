use crate::types::{new, Namespace, ObjectRef};

fn check_ok<S: Into<String>>(obj: ObjectRef, msg: S) {
    assert!(!obj.read().unwrap().is_err(), "{}", msg.into())
}

fn check_is(a: ObjectRef, b: ObjectRef) {
    assert!(a.read().unwrap().is(&*b.read().unwrap()))
}

fn check_is_not(a: ObjectRef, b: ObjectRef) {
    assert!(!a.read().unwrap().is(&*b.read().unwrap()))
}

fn check_type_is(a: ObjectRef, b: ObjectRef) {
    let a = a.read().unwrap();
    let b = b.read().unwrap();
    assert!(a.class().read().unwrap().is(&*b.class().read().unwrap()))
}

fn check_eq(a: ObjectRef, b: ObjectRef) {
    assert!(a.read().unwrap().is_equal(&*b.read().unwrap()));
}

fn check_ne(a: ObjectRef, b: ObjectRef) {
    assert!(!a.read().unwrap().is_equal(&*b.read().unwrap()));
}

fn _check_id_eq(a: ObjectRef, b: ObjectRef) {
    let a_id = a.read().unwrap().id();
    let b_id = b.read().unwrap().id();
    assert_eq!(a_id, b_id)
}

fn check_id_ne(a: ObjectRef, b: ObjectRef) {
    let a_id = a.read().unwrap().id();
    let b_id = b.read().unwrap().id();
    assert_ne!(a_id, b_id)
}

fn check_attr(obj: ObjectRef, name: &str) {
    check_ok(
        obj.clone().read().unwrap().get_attr(name, obj.clone()),
        format!("Attribute {name} is not OK"),
    )
}

fn check_attr_eq(obj: ObjectRef, name: &str, to: ObjectRef) {
    assert!(
        obj.clone()
            .read()
            .unwrap()
            .get_attr(name, obj.clone())
            .read()
            .unwrap()
            .is_equal(&*to.read().unwrap()),
        "attribute {name} not equal to {to}",
        to = to.read().unwrap()
    );
}

mod float {
    use super::*;

    #[test]
    fn test_float() {
        let float1 = new::float(0.0);
        let float2 = new::float(0.0);
        let float3 = new::float(1.0);

        check_type_is(float1.clone(), float2.clone());
        check_type_is(float2.clone(), float3.clone());

        check_is(float1.clone(), float1.clone());
        check_is_not(float1.clone(), float2.clone());
        check_is_not(float1.clone(), float3.clone());

        check_eq(float1.clone(), float2.clone());
        check_ne(float1.clone(), float3.clone());

        check_id_ne(float1.clone(), float2.clone());
        check_id_ne(float2.clone(), float3.clone());
    }

    #[test]
    fn test_compare_to_int() {
        let float = new::float(1.0);
        let int = new::int(1);
        check_eq(float.clone(), int.clone());
        check_eq(int.clone(), float.clone());
    }
}

mod list {
    use super::*;

    #[test]
    fn test_push_exists() {
        let obj_ref = new::list(vec![]);
        let list = obj_ref.read().unwrap();
        let push = list.get_attr("push", obj_ref.clone());
        check_ok(push.clone(), "list.push() is not OK");
        assert!(push.read().unwrap().is_builtin_func());
    }
}

mod custom {
    use super::*;

    #[test]
    fn test_custom() {
        let mod1 = new::builtin_module("test1", "<test1>", "test module 1", &[]);
        let mod2 = new::builtin_module("test2", "<test2>", "test module 2", &[]);

        let t1 = new::custom_type(mod1, "Custom1");

        let mut ns = Namespace::default();
        ns.add_obj("value", new::nil());
        let t1_obj1 = new::custom_instance(t1.clone(), ns);

        let mut ns = Namespace::default();
        ns.add_obj("value", new::nil());
        let t1_obj2 = new::custom_instance(t1.clone(), ns);

        let mut ns = Namespace::default();
        ns.add_obj("value", new::nil());
        let t1_obj3 = new::custom_instance(t1.clone(), ns);

        check_attr(t1.clone(), "$id");
        check_attr(t1.clone(), "$type");
        check_attr(t1_obj1.clone(), "$id");
        check_attr(t1_obj1.clone(), "$type");

        let result =
            t1_obj3.write().unwrap().set_attr("value", new::int(1), t1_obj3.clone());
        check_ok(result, "Could not set `value` on t1_obj3");
        check_attr(t1_obj3.clone(), "value");
        check_attr_eq(t1_obj3.clone(), "value", new::int(1));

        let t2 = new::custom_type(mod2, "Custom2");
        let t2_obj1 = new::custom_instance(t2, Namespace::default());

        // An object should be equal to itself.
        check_eq(t1_obj1.clone(), t1_obj1.clone());

        // An object should be equal to an object of the SAME type with
        // the same attributes.
        check_eq(t1_obj1.clone(), t1_obj2.clone());

        // An object should NOT be equal to an object of the SAME type with
        // the DIFFERENT attributes.
        check_ne(t1_obj1.clone(), t1_obj3.clone());

        // An object should NOT be equal to an object of a DIFFERENT type,
        // regardless of attributes.
        check_ne(t1_obj1.clone(), t2_obj1.clone());
    }
}
