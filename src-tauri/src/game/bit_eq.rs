use std::hash::{Hash, Hasher};

/// Bit-level equality for types that may contain f32/f64 fields.
///
/// Floating-point fields are compared via `to_bits()` rather than `==`, so NaN
/// bit patterns are handled consistently in snapshot diffing.
pub trait BitEq {
    fn bit_eq(&self, other: &Self) -> bool;
}

/// Bit-level hashing for types that may contain f32/f64 fields.
///
/// Floating-point fields hash their raw bit patterns to match `BitEq` semantics.
pub trait BitHash {
    fn bit_hash<H: Hasher>(&self, state: &mut H);
}

macro_rules! impl_eq_hash_for_hash_type {
    ($($type:ty),* $(,)?) => {
        $(
            impl BitEq for $type {
                fn bit_eq(&self, other: &Self) -> bool {
                    self == other
                }
            }

            impl BitHash for $type {
                fn bit_hash<H: Hasher>(&self, state: &mut H) {
                    self.hash(state);
                }
            }
        )*
    };
}

impl BitEq for f32 {
    fn bit_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl BitHash for f32 {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl BitEq for f64 {
    fn bit_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl BitHash for f64 {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl_eq_hash_for_hash_type!(u8, u16, u32, u64, usize, bool, String);

impl<T: BitEq> BitEq for Option<T> {
    fn bit_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(a), Some(b)) => a.bit_eq(b),
            _ => false,
        }
    }
}

impl<T: BitHash> BitHash for Option<T> {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        match self {
            None => 0u8.hash(state),
            Some(value) => {
                1u8.hash(state);
                value.bit_hash(state);
            }
        }
    }
}

impl<T: BitEq> BitEq for Vec<T> {
    fn bit_eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a.bit_eq(b))
    }
}

impl<T: BitHash> BitHash for Vec<T> {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        for item in self {
            item.bit_hash(state);
        }
    }
}
