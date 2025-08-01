/// Allows extracting bitfields from bits.
pub trait Bits<B> {
    fn extract<const START: usize, const END: usize>(self) -> B;

    fn insert<const START: usize, const END: usize>(&mut self, bits: B);
}

/// This trait allows clone-free conversion from the bit representation.
pub trait FromBits<B> {
    fn from(bits: &B) -> Self;
}

/// This trait allows clone-free conversion into the bit representation.
pub trait IntoBits<B> {
    fn into(&self) -> B;
}

impl<T, Y> IntoBits<Y> for T
where
    Y: FromBits<T>,
{
    #[inline(always)]
    fn into(&self) -> Y {
        Y::from(self)
    }
}

macro_rules! impl_conv {
    ($ty:ty: $repr:ty) => {
        impl Bits<$repr> for $ty {
            #[inline(always)]
            fn extract<const START: usize, const END: usize>(self) -> $repr {
                const {
                    assert!(START < size_of::<Self>() * 8);
                    assert!(END < size_of::<Self>() * 8);
                    assert!(START <= END);
                    assert!(END - START < size_of::<$repr>() * 8);
                }
                ((self >> START) & (1 as $ty).unbounded_shl((END + 1 - START) as u32).wrapping_sub(1)) as $repr
            }

            #[inline(always)]
            fn insert<const START: usize, const END: usize>(&mut self, bits: $repr) {
                const {
                    assert!(START < size_of::<Self>() * 8);
                    assert!(END < size_of::<Self>() * 8);
                    assert!(START <= END);
                    assert!(END - START < size_of::<Self>() * 8);
                }
                let mask = !((1 << START) - 1) & (1 as $ty).unbounded_shl((END + 1) as u32).wrapping_sub(1);
                let bits = bits as $ty;
                *self &= !mask;
                *self |= (bits << START) & mask;
            }
        }

        impl FromBits<$repr> for $ty {
            #[inline(always)]
            fn from(bits: &$repr) -> Self {
                *bits as $ty
            }
        }
    };
    ($($ty:ty),*) => {
        $(
            impl_conv!($ty: u8);
            impl_conv!($ty: u16);
            impl_conv!($ty: u32);
            impl_conv!($ty: u64);
            impl_conv!($ty: u128);
        )*
    };
}

impl_conv!(u8, u16, u32, u64, u128);

#[inline(always)]
pub fn extract<B, T: Bits<B>, R: FromBits<B>, const START: usize, const END: usize>(bits: T) -> R {
    R::from(&T::extract::<START, END>(bits))
}

#[inline(always)]
pub fn insert<B, T: Bits<B>, const START: usize, const END: usize>(
    value: &mut T,
    bits: &impl IntoBits<B>,
) {
    T::insert::<START, END>(value, bits.into());
}
