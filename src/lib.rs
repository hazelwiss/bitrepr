#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate self as bitrepr;

#[doc(hidden)]
pub mod bits;

mod impls;

pub use bitrepr_macros::{BitPack, TryBitPack, UnsafeBitPack};

/// Types which can be represented as bits use this trait to specify the type of the underlying bit representation.
///
/// For example, a `f32` would use `u32` as its bit representation, because `f34` needs *at least* 32 bits to properly
/// represent its value.
///
/// Although any type may be accepted, only unsigned integer types are useful as representation types.
pub trait BitRepr {
    /// The bit representation type of `Self`.
    ///
    /// This will be some unsigned integer in any sane implementation of this trait.
    type Repr;
}

/// Types which are representable as bits may use this trait to convert into or from their bit representation.
/// For more information, look at the crate root documentation.
///
/// This trait is generally not intended to be manually implemented, and most of the time the derive macro `#[derive(BitPack)]`
/// should be used. To see examples of using the macro look at the crate documentation.
pub trait BitPack: BitRepr {
    /// Packs a value of `Self` into its bits representable format.
    fn pack(&self) -> Self::Repr;

    /// Convert a bit representation of `Self` into an instance of `Self`.
    fn unpack(repr: Self::Repr) -> Self;
}

/// Types which are representable as bits may use this trait to convert into or from their bit representation.
/// This trait is different from [`BitPack`] in that the conversion from the bit representation may fail.
/// For more information, look at the crate root documentation.
///
/// This trait is generally not intended to be manually implemented, and most of the time the derive macro `#[derive(TryBitPack)]`
/// should be used. To see examples of using the macro look at the crate documentation.
pub trait TryBitPack: BitRepr + Sized {
    /// Packs a value of `Self` into its bits representable format.
    fn pack(&self) -> Self::Repr;

    /// Attempts to convert a bit representation of `Self` into an instance of `Self`. Returns [None] on failure.
    fn try_unpack(repr: Self::Repr) -> Option<Self>;
}

/// Types which are representable as bits may use this trait to convert into or from their bit representation.
/// This trait is different from [`BitPack`] in that the conversion is unsafe and may cause undefined behaviour.
/// For more information, look at the crate root documentation.
///
/// This trait is generally not intended to be manually implemented, and most of the time the derive macro `#[derive(UnsafeBitPack)]`
/// should be used. To see examples of using the macro look at the crate documentation.
pub trait UnsafeBitPack: BitRepr {
    /// Packs a value of `Self` into its bits representable format.
    fn pack(&self) -> Self::Repr;

    /// Convert a bit representation of `Self` into an instance of `Self`. Does not guarantee `repr` is a valid bit representation of
    /// `Self` and may potentially cause UB if `repr` is an invalid representation of `Self`. For a safe alternative, use [TryBitPack].
    ///
    /// # Safety
    ///
    /// The contract of this function has to guarantee `repr` is a valid bit representation of `Self`. This guarantee depends on `Self`'s
    /// implementation of this trait.
    unsafe fn unsafe_unpack(repr: Self::Repr) -> Self;
}

#[cfg(test)]
mod test;
