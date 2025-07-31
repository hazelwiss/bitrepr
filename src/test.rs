use crate::{BitPack, TryBitPack};

macro_rules! bits {
    ($ty:ty, $expr:expr, $start:expr, $end:expr) => {
        ($expr >> $start) & ((1 as $ty).unbounded_shl($end + 1 - $start)).wrapping_sub(1)
    };
    ($ty:ty, $expr:expr, $bit:expr) => {
        bits!($ty, $expr, $bit, $bit) != 0
    };
}

macro_rules! value {
    ($ty:ty, $start:expr, $end:expr, $value:expr) => {
        ($value & ((1 as $ty).unbounded_shl($end + 1)).wrapping_sub(1)) << $start
    };
}

macro_rules! masked_eq {
    ($ty:ty, $l:expr, $r:expr, [$($start:literal..$end:literal),*]) => {
        $(
            (bits!($ty, $l, $start, $end) == bits!($ty, $r, $start, $end))
        )|*
    };
}

#[test]
fn transitive() {
    #[derive(BitPack, Debug)]
    #[bitpack(u32)]
    struct Simple {
        #[bitpack(0..8)]
        f0: u8,
        #[bitpack(8..=23)]
        f1: u16,
        #[bitpack(24)]
        f2: bool,
        #[bitpack(26)]
        f3: bool,
        #[bitpack(29..)]
        f4: u8,
    }

    for v in 0..u32::MAX {
        let f0 = bits!(u32, v, 0, 7);
        let f1 = bits!(u32, v, 8, 23);
        let f2 = bits!(u32, v, 24);
        let f3 = bits!(u32, v, 26);
        let f4 = bits!(u32, v, 29, 31);

        let simple = Simple::unpack(v);

        assert_eq!(simple.f0 as u32, f0);
        assert_eq!(simple.f1 as u32, f1);
        assert_eq!(simple.f2, f2);
        assert_eq!(simple.f3, f3);
        assert_eq!(simple.f4 as u32, f4);

        let packed = BitPack::pack(&simple);

        assert!(masked_eq!(
            u32,
            v,
            packed,
            [0..7, 8..23, 24..24, 26..26, 29..31]
        ));
        assert!(masked_eq!(u32, 0, packed, [25..25, 27..28]))
    }
}

#[test]
fn unpacks() {
    #[derive(BitPack, Debug)]
    #[bitpack(u32)]
    struct Packed {
        #[bitpack(7)]
        f0: bool,
        #[bitpack(11..=17)]
        f1: u8,
        #[bitpack(21..)]
        f2: u32,
    }

    for (f0, f1, f2) in [(true, 0x7f, 0x7ff), (true, 0, 0)] {
        let packed =
            value!(u32, 7, 7, f0 as u32) | value!(u32, 11, 18, f1 as u32) | value!(u32, 21, 31, f2);
        let packed = Packed::unpack(packed);

        assert_eq!(f0, packed.f0);
        assert_eq!(f1, packed.f1);
        assert_eq!(f2, packed.f2);
    }

    for ((f0, f1, f2), (cf0, cf1, cf2)) in [
        ((false, 0xff, 0xfff), (false, 0x7f, 0x7ff)),
        ((true, u32::MAX, u32::MAX), (true, 0x7f, 0x7ff)),
    ] {
        let packed =
            value!(u32, 7, 31, f0 as u32) | value!(u32, 11, 31, f1) | value!(u32, 21, 31, f2);
        let packed = Packed::unpack(packed);

        assert_eq!(cf0, packed.f0);
        assert_eq!(cf1, packed.f1);
        assert_eq!(cf2, packed.f2);
    }
}

#[test]
fn packs() {
    #[derive(BitPack, Debug)]
    #[bitpack(u32)]
    struct UnPacked {
        #[bitpack(7)]
        f0: bool,
        #[bitpack(11..=17)]
        f1: u8,
        #[bitpack(21..)]
        f2: u32,
    }

    for (unpacked, (cf0, cf1, cf2)) in [
        (
            UnPacked {
                f0: true,
                f1: 0x78,
                f2: 0xffff,
            },
            (true, 0x78, 0x7ff),
        ),
        (
            UnPacked {
                f0: true,
                f1: 0x78,
                f2: 0xffff,
            },
            (true, 0x78, 0x7ff),
        ),
    ] {
        let pack = BitPack::pack(&unpacked);

        assert_eq!(cf0, bits!(u32, pack, 7));
        assert_eq!(cf1, bits!(u32, pack, 11, 17));
        assert_eq!(cf2, bits!(u32, pack, 21, 31));
    }
}

#[test]
fn full() {
    #[derive(BitPack, Debug)]
    #[bitpack(u32)]
    struct FullU32 {
        #[bitpack(..)]
        full: u32,
    }

    for v in 0..u32::MAX {
        let full = FullU32 { full: v };
        let pack = BitPack::pack(&full);
        assert_eq!(v, pack);
        let full = FullU32::unpack(v);
        assert_eq!(full.full, v);
    }

    #[derive(BitPack, Debug)]
    #[bitpack(u16)]
    struct FullU16 {
        #[bitpack(..)]
        full: u16,
    }

    for v in 0..u16::MAX {
        let full = FullU16 { full: v };
        let pack = BitPack::pack(&full);
        assert_eq!(v, pack);
        let full = FullU16::unpack(v);
        assert_eq!(full.full, v);
    }

    #[derive(BitPack, Debug)]
    #[bitpack(u8)]
    struct FullU8 {
        #[bitpack(..)]
        full: u8,
    }

    for v in 0..u8::MAX {
        let full = FullU8 { full: v };
        let pack = BitPack::pack(&full);
        assert_eq!(v, pack);
        let full = FullU8::unpack(v);
        assert_eq!(full.full, v);
    }
}

#[test]
fn try_transitive() {
    #[derive(TryBitPack, Debug)]
    #[bitpack(u32)]
    struct Simple {
        #[bitpack(0..8)]
        f0: u8,
        #[bitpack(8..=23)]
        f1: u16,
        #[bitpack(24)]
        f2: bool,
        #[bitpack(26)]
        f3: bool,
        #[bitpack(29..)]
        f4: u8,
    }

    for v in 0..u32::MAX {
        let f0 = bits!(u32, v, 0, 7);
        let f1 = bits!(u32, v, 8, 23);
        let f2 = bits!(u32, v, 24);
        let f3 = bits!(u32, v, 26);
        let f4 = bits!(u32, v, 29, 31);

        let simple = Simple::try_unpack(v).unwrap();

        assert_eq!(simple.f0 as u32, f0);
        assert_eq!(simple.f1 as u32, f1);
        assert_eq!(simple.f2, f2);
        assert_eq!(simple.f3, f3);
        assert_eq!(simple.f4 as u32, f4);

        let packed = simple.pack();

        assert!(masked_eq!(
            u32,
            v,
            packed,
            [0..7, 8..23, 24..24, 26..26, 29..31]
        ));
        assert!(masked_eq!(u32, 0, packed, [25..25, 27..28]))
    }
}

#[test]
fn try_unpacks() {
    #[derive(TryBitPack, Debug)]
    #[bitpack(u32)]
    struct Packed {
        #[bitpack(7)]
        f0: bool,
        #[bitpack(11..=17)]
        f1: u8,
        #[bitpack(21..)]
        f2: u32,
    }

    for (f0, f1, f2) in [(true, 0x7f, 0x7ff), (true, 0, 0)] {
        let packed =
            value!(u32, 7, 7, f0 as u32) | value!(u32, 11, 18, f1 as u32) | value!(u32, 21, 31, f2);
        let packed = Packed::try_unpack(packed).unwrap();

        assert_eq!(f0, packed.f0);
        assert_eq!(f1, packed.f1);
        assert_eq!(f2, packed.f2);
    }

    for ((f0, f1, f2), (cf0, cf1, cf2)) in [
        ((false, 0xff, 0xfff), (false, 0x7f, 0x7ff)),
        ((true, u32::MAX, u32::MAX), (true, 0x7f, 0x7ff)),
    ] {
        let packed =
            value!(u32, 7, 31, f0 as u32) | value!(u32, 11, 31, f1) | value!(u32, 21, 31, f2);
        let packed = Packed::try_unpack(packed).unwrap();

        assert_eq!(cf0, packed.f0);
        assert_eq!(cf1, packed.f1);
        assert_eq!(cf2, packed.f2);
    }
}

#[test]
fn try_packs() {
    #[derive(TryBitPack, Debug)]
    #[bitpack(u32)]
    struct UnPacked {
        #[bitpack(7)]
        f0: bool,
        #[bitpack(11..=17)]
        f1: u8,
        #[bitpack(21..)]
        f2: u32,
    }

    for (unpacked, (cf0, cf1, cf2)) in [
        (
            UnPacked {
                f0: true,
                f1: 0x78,
                f2: 0xffff,
            },
            (true, 0x78, 0x7ff),
        ),
        (
            UnPacked {
                f0: true,
                f1: 0x78,
                f2: 0xffff,
            },
            (true, 0x78, 0x7ff),
        ),
    ] {
        let pack = unpacked.pack();

        assert_eq!(cf0, bits!(u32, pack, 7));
        assert_eq!(cf1, bits!(u32, pack, 11, 17));
        assert_eq!(cf2, bits!(u32, pack, 21, 31));
    }
}

#[test]
fn try_full() {
    #[derive(TryBitPack, Debug)]
    #[bitpack(u32)]
    struct FullU32 {
        #[bitpack(..)]
        full: u32,
    }

    for v in 0..u32::MAX {
        let full = FullU32 { full: v };
        let pack = full.pack();
        assert_eq!(v, pack);
        let full = FullU32::try_unpack(v).unwrap();
        assert_eq!(full.full, v);
    }

    #[derive(TryBitPack, Debug)]
    #[bitpack(u16)]
    struct FullU16 {
        #[bitpack(..)]
        full: u16,
    }

    for v in 0..u16::MAX {
        let full = FullU16 { full: v };
        let pack = full.pack();
        assert_eq!(v, pack);
        let full = FullU16::try_unpack(v).unwrap();
        assert_eq!(full.full, v);
    }

    #[derive(TryBitPack, Debug)]
    #[bitpack(u8)]
    struct FullU8 {
        #[bitpack(..)]
        full: u8,
    }

    for v in 0..u8::MAX {
        let full = FullU8 { full: v };
        let pack = full.pack();
        assert_eq!(v, pack);
        let full = FullU8::try_unpack(v).unwrap();
        assert_eq!(full.full, v);
    }
}

#[test]
fn complex_types() {
    struct ComplexString(String);

    impl BitPack for ComplexString {
        type Repr = u8;

        fn pack(&self) -> Self::Repr {
            self.0.as_str().parse::<u8>().unwrap()
        }

        fn unpack(repr: Self::Repr) -> Self {
            Self(repr.to_string())
        }
    }

    #[derive(BitPack)]
    #[bitpack(u128)]
    struct Complex {
        #[bitpack(..=31)]
        f0: f32,
        #[bitpack(32..=63)]
        f1: f64,
        #[bitpack(64..=71)]
        f2: char,
        #[bitpack(72..=79)]
        f3: ComplexString,
    }

    // Test that unit struct declaration compiles. There should be no difference in the logic though.
    #[expect(unused)]
    #[derive(BitPack)]
    #[bitpack(u64)]
    struct Test(
        #[bitpack(0)] u32,
        #[bitpack(2)] bool,
        #[bitpack(skip)] u64,
        #[bitpack(14..61)] u64,
        #[bitpack(62..)] u8,
    );
}

#[test]
fn try_complex_types() {
    // Test that unit struct declaration compiles. There should be no difference in the logic though.
    #[expect(unused)]
    #[derive(TryBitPack)]
    #[bitpack(u64)]
    struct Test(
        #[bitpack(0)] u32,
        #[bitpack(2)] bool,
        #[bitpack(skip)] u64,
        #[bitpack(14..61)] u64,
        #[bitpack(62..)] u8,
    );
}
