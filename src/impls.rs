use crate::{BitPack, TryBitPack};

macro_rules! impl_bitpacks {
    ($($ty:ty),*) => {
        $(
            impl BitPack for $ty {
                type Repr = $ty;

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

impl BitPack for bool {
    type Repr = u8;

    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        *self as u8
    }

    #[inline(always)]
    fn unpack(repr: Self::Repr) -> Self {
        repr != 0
    }
}

impl BitPack for f32 {
    type Repr = u32;

    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        f32::to_bits(*self)
    }

    #[inline(always)]
    fn unpack(repr: Self::Repr) -> Self {
        f32::from_bits(repr)
    }
}

impl BitPack for f64 {
    type Repr = u64;

    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        f64::to_bits(*self)
    }

    #[inline(always)]
    fn unpack(repr: Self::Repr) -> Self {
        f64::from_bits(repr)
    }
}

impl TryBitPack for char {
    type Repr = u32;

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
    type Repr = T::Repr;

    #[inline(always)]
    fn pack(&self) -> Self::Repr {
        BitPack::pack(self)
    }

    #[inline(always)]
    fn try_unpack(repr: Self::Repr) -> Option<Self> {
        Some(BitPack::unpack(repr))
    }
}
