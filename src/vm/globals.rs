//! This module defines global constants that are shared across all
//! code units. These can (and should) be loaded by index in the VM
//! using dedicated instructions (`LOAD_NIL`, `LOAD_GLOBAL_CONST`, etc).

use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};

use once_cell::sync::Lazy;

use crate::types::gen::{obj_ref, obj_ref_t};
use crate::types::ObjectRef;

use crate::types::always::Always;
use crate::types::bool::Bool;
use crate::types::int::Int;
use crate::types::nil::Nil;
use crate::types::str::Str;
use crate::types::tuple::Tuple;

pub static NIL: Lazy<obj_ref_t!(Nil)> = Lazy::new(|| obj_ref!(Nil::new()));
pub static TRUE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(true)));
pub static FALSE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(false)));
pub static ALWAYS: Lazy<obj_ref_t!(Always)> = Lazy::new(|| obj_ref!(Always::new()));

pub static EMPTY_STR: Lazy<obj_ref_t!(Str)> =
    Lazy::new(|| obj_ref!(Str::new("".to_owned())));

pub static EMPTY_TUPLE: Lazy<obj_ref_t!(Tuple)> =
    Lazy::new(|| obj_ref!(Tuple::new(vec![])));

pub static SHARED_INT_MAX: usize = 256;
pub static SHARED_INT_MAX_BIGINT: Lazy<BigInt> =
    Lazy::new(|| BigInt::from(SHARED_INT_MAX));
pub static SHARED_INTS: Lazy<Vec<obj_ref_t!(Int)>> = Lazy::new(|| {
    (0..=SHARED_INT_MAX).map(|i| obj_ref!(Int::new(BigInt::from(i)))).collect()
});

pub const NIL_INDEX: usize = 0;
pub const TRUE_INDEX: usize = 1;
pub const FALSE_INDEX: usize = 2;
pub const ALWAYS_INDEX: usize = 3;
pub const EMPTY_STR_INDEX: usize = 4;
pub const EMPTY_TUPLE_INDEX: usize = 5;
pub const SHARED_INT_INDEX: usize = 6;

/// Get the global constants.
///
/// NOTE: This is only intended to be called _once_.
pub fn get_global_constants() -> Vec<ObjectRef> {
    let mut global_constants: Vec<ObjectRef> = vec![
        NIL.clone(),
        TRUE.clone(),
        FALSE.clone(),
        ALWAYS.clone(),
        EMPTY_STR.clone(),
        EMPTY_TUPLE.clone(),
    ];
    for int in SHARED_INTS.iter() {
        global_constants.push(int.clone());
    }
    global_constants
}

/// Get the global constant index for the `int` if it's in the shared
/// int range.
pub fn shared_int_index(int: &BigInt) -> Option<usize> {
    if int.is_zero() {
        Some(SHARED_INT_INDEX)
    } else if int.is_positive() && int <= Lazy::force(&SHARED_INT_MAX_BIGINT) {
        Some(int.to_usize().unwrap() + SHARED_INT_INDEX)
    } else {
        None
    }
}

/// Get the global constant at `index`.
///
/// NOTE: This is only intended for use in testing.
pub(crate) fn get_global_constant(index: usize) -> Option<ObjectRef> {
    let global_constants = get_global_constants();
    global_constants.get(index).cloned()
}
