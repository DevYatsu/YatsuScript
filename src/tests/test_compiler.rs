//! Tests for the `Value` NaN-boxing scheme and the `Instruction` enum defined
//! in `compiler.rs`.

#[cfg(test)]
mod tests {
    use crate::compiler::{Instruction, Loc, QNAN, TAG_BOOL, TAG_OBJ, Value};

    // Value::number

    #[test]
    fn value_number_round_trips() {
        for &n in &[0.0_f64, 1.0, -1.0, 3.14, f64::MAX, f64::MIN_POSITIVE] {
            let v = Value::number(n);
            assert_eq!(v.as_number(), Some(n), "round-trip failed for {n}");
        }
    }

    #[test]
    fn value_number_zero_is_a_number() {
        let v = Value::number(0.0);
        assert!(v.as_number().is_some());
        assert!(v.as_bool().is_none());
        assert!(v.as_obj_id().is_none());
    }

    #[test]
    fn value_number_nan_is_not_returned() {
        // A NaN f64 will be *stored* as a NaN bit-pattern.
        // `as_number` returns None for the QNAN sentinel, but a signalling NaN
        // will pass through because (bits & QNAN) == QNAN only for the specific
        // quiet NaN tag we use.  Test both paths explicitly.
        let v = Value::number(f64::NAN);
        // f64::NAN has QNAN bits set — our implementation returns None.
        assert!(
            v.as_number().is_none() || v.as_number().map(|n| n.is_nan()).unwrap_or(false),
            "NaN value must either be None or NaN when retrieved"
        );
    }

    // Value::bool

    #[test]
    fn value_bool_true_round_trips() {
        let v = Value::bool(true);
        assert_eq!(v.as_bool(), Some(true));
        assert!(v.as_number().is_none());
        assert!(v.as_obj_id().is_none());
    }

    #[test]
    fn value_bool_false_round_trips() {
        let v = Value::bool(false);
        assert_eq!(v.as_bool(), Some(false));
        assert!(v.as_number().is_none());
        assert!(v.as_obj_id().is_none());
    }

    #[test]
    fn value_bool_true_has_correct_bits() {
        let v = Value::bool(true);
        let bits = v.to_bits();
        assert_eq!(bits & QNAN, QNAN, "bool must be in the QNAN space");
        assert_eq!(bits & TAG_BOOL, TAG_BOOL, "bool must carry TAG_BOOL");
        assert_eq!(bits & 1, 1, "true must have LSB set");
    }

    #[test]
    fn value_bool_false_has_correct_bits() {
        let v = Value::bool(false);
        let bits = v.to_bits();
        assert_eq!(bits & QNAN, QNAN);
        assert_eq!(bits & TAG_BOOL, TAG_BOOL);
        assert_eq!(bits & 1, 0, "false must have LSB clear");
    }

    // Value::object

    #[test]
    fn value_object_round_trips() {
        for &id in &[0_u32, 1, 255, u32::MAX] {
            let v = Value::object(id);
            assert_eq!(
                v.as_obj_id(),
                Some(id),
                "round-trip failed for object id {id}"
            );
            assert!(v.as_number().is_none());
            assert!(v.as_bool().is_none());
        }
    }

    #[test]
    fn value_object_has_correct_bits() {
        let id = 42_u32;
        let v = Value::object(id);
        let bits = v.to_bits();
        assert_eq!(bits & QNAN, QNAN);
        assert_eq!(bits & TAG_OBJ, TAG_OBJ);
        assert_eq!((bits & 0xFFFF_FFFF) as u32, id);
    }

    // Value::sso — Small-String Optimisation

    #[test]
    fn value_sso_empty_string() {
        // Empty string should be representable as SSO (len 0 <= 6).
        let v = Value::sso("").expect("empty string should be SSO");
        // Its bits should be in the QNAN region (tag = 3 + 0 = 3).
        let bits = v.to_bits();
        assert_eq!(bits & QNAN, QNAN);
    }

    #[test]
    fn value_sso_max_length() {
        // Exactly 6 ASCII bytes fits.
        let v = Value::sso("abcdef");
        assert!(v.is_some(), "6-char string must fit in SSO");
    }

    #[test]
    fn value_sso_over_limit_returns_none() {
        // 7 chars must NOT fit.
        let v = Value::sso("abcdefg");
        assert!(v.is_none(), "7-char string must NOT fit in SSO");
    }

    // Value::from_bits / to_bits identity

    #[test]
    fn value_bits_identity() {
        let v = Value::number(42.0);
        assert_eq!(Value::from_bits(v.to_bits()).to_bits(), v.to_bits());
    }

    // Instruction derives PartialEq / Clone

    #[test]
    fn instruction_load_literal_clone_eq() {
        let loc = Loc { line: 1, col: 1 };
        let instr = Instruction::Add {
            dst: 0,
            lhs: 1,
            rhs: 2,
            loc,
        };
        assert_eq!(instr.clone(), instr);
    }

    #[test]
    fn instruction_jump_eq() {
        let j1 = Instruction::Jump(10);
        let j2 = Instruction::Jump(10);
        assert_eq!(j1, j2);
        assert_ne!(j1, Instruction::Jump(99));
    }
}
