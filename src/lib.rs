extern crate self as bitrepr;

#[doc(hidden)]
pub mod bits;

mod impls;

pub use bitrepr_macros::{BitPack, TryBitPack, UnsafeBitPack};

pub trait BitRepr {
    type Repr;
}

/// Types which are representable as bits may use this trait to convert into or from their bit representation.
///
/// This trait is generally not intended to be manually implemented, and most of the time the derive macro `#[derive(BitPack)]`
/// should be used. To see examples of using the macro look at the crate documentation.
pub trait BitPack: BitRepr {
    /// Packs a value of `Self` into its bits representable format.
    fn pack(&self) -> Self::Repr;

    /// Convers a bit representation of `Self` into an instance of `Self`.
    fn unpack(repr: Self::Repr) -> Self;
}

/// Types which are representable as bits may use this trait to convert into or from their bit representation.
/// This trait is different from [`BitPack`] in that the conversion from the bit representation may fail.
///
/// This trait is generally not intended to be manually implemented, and most of the time the derive macro `#[derive(TryBitPack)]`
/// should be used. To see examples of using the macro look at the crate documentation.
pub trait TryBitPack: BitRepr + Sized {
    fn pack(&self) -> Self::Repr;

    fn try_unpack(repr: Self::Repr) -> Option<Self>;
}

/// Types which are representable as bits may use this trait to convert into or from their bit representation.
/// This trait is different from [`BitPack`] in that the conversion is unsafe and may cause undefined behaviour.
///
/// This trait is generally not intended to be manually implemented, and most of the time the derive macro `#[derive(UnsafeBitPack)]`
/// should be used. To see examples of using the macro look at the crate documentation.
pub trait UnsafeBitPack: BitRepr {
    fn pack(&self) -> Self::Repr;

    /// # Safety
    ///
    /// When unpacking
    unsafe fn unsafe_unpack(repr: Self::Repr) -> Self;
}

#[cfg(test)]
mod test;
