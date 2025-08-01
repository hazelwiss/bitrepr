use crate::{BitPack, BitRepr, TryBitPack, UnsafeBitPack};

macro_rules! impl_bitpacks {
    ($($ty:ty),*) => {
        $(
            impl BitRepr for $ty {
                type Repr = $ty;
            }

            impl BitPack for $ty {
                #[inline(always)]
                fn pack(&self) -> Self::Repr {
                    *self
                }

                #[inline(always)]
                fn unpack(repr: Self::Repr) -> Self {
                    repr
                }
            }
        )*
    };
}

impl_bitpacks!(u8, u16, u32, u64, u128);

impl BitRepr for bool {
    type Repr = u8;
}

impl BitPack for bool {
    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        *self as u8
    }

    #[inline(always)]
    fn unpack(repr: Self::Repr) -> Self {
        repr != 0
    }
}

impl BitRepr for f32 {
    type Repr = u32;
}

impl BitPack for f32 {
    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        f32::to_bits(*self)
    }

    #[inline(always)]
    fn unpack(repr: Self::Repr) -> Self {
        f32::from_bits(repr)
    }
}

impl BitRepr for f64 {
    type Repr = u64;
}

impl BitPack for f64 {
    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        f64::to_bits(*self)
    }

    #[inline(always)]
    fn unpack(repr: Self::Repr) -> Self {
        f64::from_bits(repr)
    }
}

impl BitRepr for char {
    type Repr = u32;
}

impl TryBitPack for char {
    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        *self as u32
    }

    #[inline(always)]
    fn try_unpack(repr: Self::Repr) -> Option<Self> {
        char::from_u32(repr)
    }
}

impl<T: BitPack> TryBitPack for T {
    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        BitPack::pack(self)
    }

    #[inline(always)]
    fn try_unpack(repr: Self::Repr) -> Option<Self> {
        Some(BitPack::unpack(repr))
    }
}

// This implementation will also apply for types which implement `BitPack`
// because types implementing `BitPack` implicitly implement `TryBitPack`.
impl<T: TryBitPack> UnsafeBitPack for T {
    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        TryBitPack::pack(self)
    }

    #[inline(always)]
    unsafe fn unsafe_unpack(repr: Self::Repr) -> Self {
        // SAFETY: Every type which implements TryBitPack implictly
        // implements UnsafeBitPack assuming the unpack was a success.
        // It is up to the caller to guarantee this promise is not broken,
        // hence at this point we can assume it is safe to unwrap.
        unsafe { TryBitPack::try_unpack(repr).unwrap_unchecked() }
    }
}
