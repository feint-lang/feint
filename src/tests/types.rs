use crate::types::{new, Namespace, ObjectTrait};

mod float {
    use super::*;

    #[test]
    fn test_float() {
        let float1 = new::float(0.0);
        let float2 = new::float(0.0);
        let float3 = new::float(1.0);

        let float1 = float1.read().unwrap();
        let float2 = float2.read().unwrap();
        let float3 = float3.read().unwrap();

        assert!(float1.class().read().unwrap().is(&*float2.class().read().unwrap()));
        assert!(float2.class().read().unwrap().is(&*float3.class().read().unwrap()));

        assert!(float1.is(&*float1));
        assert!(!float1.is(&*float2));
        assert!(!float1.is(&*float3));

        assert!(float1.is_equal(&*float2));
        assert!(!float1.is_equal(&*float3));

        assert_ne!(float1.id(), float2.id());
        assert_ne!(float2.id(), float3.id());
    }

    #[test]
    fn test_compare_to_int() {
        let float = new::float(1.0);
        let int = new::int(1);
        assert!(float.read().unwrap().is_equal(&*int.read().unwrap()));
        assert!(int.read().unwrap().is_equal(&*float.read().unwrap()));
    }
}

mod list {
    use super::*;

    #[test]
    fn test_push_exists() {
        let obj_ref = new::list(vec![]);
        let list = obj_ref.read().unwrap();
        let push = list.get_attr("push");
        assert!(push.is_ok());
    }
}

mod custom {
    use super::*;

    #[test]
    fn test_custom() {
        let mod1 = new::builtin_module("test1", Namespace::new());

        let t1 = new::custom_type(mod1, "Custom1");

        let mut ns = Namespace::new();
        ns.add_obj("value", new::nil());
        let t1_obj1 = new::custom_instance(t1.clone(), ns);

        let mut ns = Namespace::new();
        ns.add_obj("value", new::nil());
        let t1_obj2 = new::custom_instance(t1.clone(), ns);

        let mut ns = Namespace::new();
        ns.add_obj("value", new::nil());
        let t1_obj3 = new::custom_instance(t1.clone(), ns);

        assert!(t1.clone().read().unwrap().get_attr("$id").is_ok());
        assert!(t1.clone().read().unwrap().get_attr("$type").is_ok());
        assert!(t1_obj1.read().unwrap().get_attr("$id").is_ok());
        assert!(t1_obj1.read().unwrap().get_attr("$type").is_ok());

        let was_set = t1_obj3.write().unwrap().set_attr("value", new::int(1));
        assert!(was_set.is_ok(), "Could not set `value` on t1_obj3");
        assert!(t1_obj3.read().unwrap().get_attr("value").is_ok());
        assert!(t1_obj3
            .read()
            .unwrap()
            .get_attr("value")
            .unwrap()
            .read()
            .unwrap()
            .is_equal(&*new::int(1).read().unwrap()));

        let mod2 = new::builtin_module("test2", Namespace::new());

        let t2 = new::custom_type(mod2, "Custom2");
        let t2_obj1 = new::custom_instance(t2, Namespace::new());

        // An object should be equal to itself.
        assert!(t1_obj1.read().unwrap().is_equal(&*t1_obj1.read().unwrap()));

        // An object should be equal to an object of the SAME type with
        // the same attributes.
        assert!(t1_obj1.read().unwrap().is_equal(&*t1_obj2.read().unwrap()));

        // An object should NOT be equal to an object of the SAME type with
        // the DIFFERENT attributes.
        assert!(!t1_obj1.read().unwrap().is_equal(&*t1_obj3.read().unwrap()));

        // An object should NOT be equal to an object of a DIFFERENT type,
        // regardless of attributes.
        assert!(!t1_obj1.read().unwrap().is_equal(&*t2_obj1.read().unwrap()));
    }
}
