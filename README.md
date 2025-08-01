[bitrepr] provides traits and derive macros for converting any type from and into a string of bits or a "bit representation". This can be useful for avoiding the use of
bitfields whenever a specific value has to exist as a bit representation, as bitfields can incur a performance penalty whenever a field has to be accessed. [bitrepr]
instead stores types normally but provides traits for converting to/from the bit representation whenver necessary, potentially granting a boost to performance and
prevents unergonomic bitfields.

The processes of converting any value to and from a string of bits is called bit packing and bit unpacking respectively.

**IF YOU WANT TO SEE EXAMPLES OF DERIVE MACROS BEING USED, IT IS AT THE BOTTOM OF THIS TEXT.**

## How does it work?

[bitrepr] provides four fundamental traits. Three of these traits have an associated derive macro, and the remaining trait is implictly implemented in those derive macros.

# BitRepr

[BitRepr] is a primitive trait that simply exist to be the single source of truth for any type if they have any bit representable type. Such as [u32] for [char] or [u64]
for [f64].

This trait is rarely relevant to the user but is required to be implented if one wants to avoid using the derive macros for a manual implementation.

# BitPack

[BitPack] is the standard bit packing/unpacking trait. It provides the methods [pack](BitPack::pack) and [unpack](BitPack::unpack). These methods are infallible under normal
circumstances, although they may panic, however a typical implementation of this trait should not panic. If the conversion is fallible, prefer to use `[TryBitPack]`.

[pack](BitPack::pack) can be used to convert from `&self` into `<Self as BitRepr>::Repr`.
[unpack](BitPack::unpack) can be used to convert from `<Self as BitRepr>::Repr` into `Self`.

[BitPack] requires the trait [BitRepr] to be implemented.

# TryBitPack

[TryBitPack] is the trait which allows fallible bit unpacking and infallible bit packing. It provides the methods [pack](TryBitPack::pack) and [try_unpack](TryBitPack::try_unpack).
Much like [BitPack] it provides a method [pack](TryBitPack) which functions equivalently as the one in [BitPack]. The difference is in unpacking, where [TryBitPack] proovides the
[try_unpack](TryBitPack::try_unpack) method which returns an `Option<Self>`. If the value being returned is [None], then the conversion failed, otherwise it was a success.

[pack](TryBitPack::pack) can be used to convert from `&self` into `<Self as BitRepr>::Repr`.
[try_unpack](TryBitPack::try_unpack) can be used to fallibly convert from `<Self as BitRepr>::Repr` into `Self`.

Every type which implements [BitPack] implicitly implements this trait by always returning `Some` for [try_unpack](TryBitPack::try_unpack).

[TryBitPack] requires the trait [BitRepr] to be implemented.

# UnsafeBitPack

[UnsafeBitPack] is the trait which allows unsafe bit unpacking. It also provides safe bit packing similar to the other traits. It provides the methods [pack](UnsafeBitPack::pack)
and [unsafe_unpack](UnsafeBitPack::unsafe_unpack). Much like [BitPack] and [TryBitPack] it provides a method [pack](UnsafeBitPack::pack) which functions equivalently as both the ones
in [BitPack] and [TryBitPack]. Just like with [TryBitPack], the difference is the unpacking, where [UnsafeBitPack] provides the [unsafe_unpack](UnsafeBitPack::unsafe_unpack) method
for allowing potentially dangerous bit unpacking. An example of this could be attempting to cast bits to an enum using [transmute](core::mem::transmute) directly for performance.
That operation is normally unsafe, however, if you guarantee that the value in the representation corresponding to that enum always corresponds to a discriminant, then this is well
defined behaviour.

[pack](UnsafeBitPack::pack) can be used to convert from `&self` into `<Self as BitRepr>::Repr`.
[try_unpack](UnsafeBitPack::unsafe_unpack) can be used to convert from `<Self as BitRepr>::Repr` into `Self` without safety guarantees.

Every type which implements [TryBitPack] implicitly implements this trait by assuming the return value of [try_unpack](TryBitPack::try_unpack) to be `Some`.
Given that all types which implement [BitPack] also implement [TryBitPack], this means all types which implement [BitPack] also implement [UnsafeBitPack]. In this case, [UnsafeBitPack]
is always safe to unpack.

[UnsafeBitPack] requires the trait [BitRepr] to be implemented.

## Derive macros

[bitrepr] does not normally require you to manually implement any of the above traits. In the spirit of convenience, the traits [BitPack], [TryBitPack] and [UnsafeBitPack] all
provide their respective derive macros. The macros are responsible for sanetizing your input (making sure you do anything strange like making two fields use the same bits) and
providing conveniences to make [bitrepr] versatile.

All macros have some common logic between them, and are distinct only in a few ways. The distinctions will be provided below.

The derive macros make use of the `#[bitpack]` attribute. Whenever you derive with any of the macros, you are required to provide a `#[bitpack(T)]` where `T` corresponds to a bit
representable type. At the moment, this should be limited to `u8`, `u16`, `u32`, `u64` and `u128`. No further input to this usage of the attribute is supported.

Every field in the derived item must also have a `#[bitpack]` attribute.
The structure being detailed below for specifying for every field is not enforced by the crate at this moment. However, this may change in the future and it is important to respect
the convention laid out here, even if it is not strictly enforced at the moment.

A `#[bitpack]` attribute on a field may have the following structure:
`#[bitpack(skip $(, $skip_meta)?)]`: The field is skipped (ignored). This requires that the type of the field implement `Default`, or `$skip_meta` provides a default value/function. 
`#[bitpack($($meta,)? $bits)]`: The field will span over `$bits` and use the options specified by `meta`.

The difference between the different derive macros simply relate to what trait they implement. This affects what fields may be used inside of the types when deriving (a [TryBitPack]
field may not be specified if deriving [BitPack] unless the `unwrap` flag is enabled for that field), as well as change the type of functions passed in with `pack_fn` and `unpack_fn`.

`meta`: Meta provides multiple "options" that may be specified together with each field. The options are:
- `unwrap`: Requires that the field implements [TryBitPack]. Will invoke [TryBitPack]} methods on the field when converting to bit representation, and will unwrap on unpacking.
- `unsafe`: Requires that the field implements [UnsafeBitPack]. Will invoke [UnsafeBitPack] methods on the field when converting to bit representation, and will assume it is safe.
- `pack_fn $expr`: uses `$expr` as a hop in for the trait's pack function. Is affected by `unwrap` and `unsafe` as it changes the reuirements of the function. Works with a capture. 
- `unpack_fn $expr`: uses `$expr` as a hop in for the trait's unpack function. Is affected by `unwrap` and `unsafe` as it changes the reuirements of the function. Works with a capture. 

`skip_meta`: May either be `default $expr` or `default_fn $expr` where `default` corresponds to a default value and `default_fn` corresponds to a default function. This is used
when unpacking and the field with this attribute has to be constructed.

`bits`: A rust range expression or integer.

`expr`: A rust expression.

## Derive macro usage

A bitpacked type may be easily specified like the following: 
```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Type {
  /// A simple boolean that occupies bit 0.
  #[bitpack(0)]
  field0: bool,
  /// A u32 spanning from bits 5 to 10 (inclusive).
  #[bitpack(5..=10)]
  field1: u32,
  /// A u32 spanning from bits 15 to 20 (exclusive).
  #[bitpack(15..20)]
  field2: u32,
}
```

```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Type {
  /// This u32 begins at 0, and ends at 15 (inclusive).
  #[bitpack(..=15)]
  half0: u32,
  /// This u32 begins at 16, and ends at 31 (inclusive).
  #[bitpack(16..)]
  half1: u32,
}
```

```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Type {
  /// Covers the entire range of the u32 representation, from bit 0 to bit 31 inclusive.
  #[bitpack(..)]
  full: u32,
}
```

A bitpack type may provide a set of different options.

The unwrap option allows us to use fallible types in [BitPack].
```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Type {
  /// the conversion from u32 to char may fail, by specifying unwrap we panic if it fails.
  #[bitpack(unwrap, ..)]
  ch: char,
}
```
or, more dangerously, the unsafe option allows you to assume the conversion never fails.
```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Type {
  /// the conversion from u32 to char may fail, by specifying unsafe, we assume the conversion never fails and the char is always valid.
  #[bitpack(unsafe, ..)]
  ch: char,
}
```

[bitrepr] allows you to provide your own functions for packing/unpacking.
```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Unpacking {
  /// Using our custom function handler, we can convert into char without panic or unsafe shenanigans.
  #[bitpack(unpack_fn |value| char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER) as u32, ..)]
  ch: u32,
}

#[derive(BitPack)]
#[bitpack(u32)]
struct Packing {
  /// Using our custom function handler, we can convert into char without panic or unsafe shenanigans.
  #[bitpack(pack_fn |value| *value as u16 as u32, ..)]
  ch: u32,
}

#[derive(BitPack)]
#[bitpack(u32)]
struct Both {
  /// Both a pack and unpack method may be provided for a single field.
  #[bitpack(
    pack_fn |value| *value as u16 as u32,
    unpack_fn |value| char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER),
    ..
  )]
  ch: char,
}
```
there is no requirement to use closures.
```
# use bitrepr::BitPack;

fn unpack(value: u32) -> u32 {
  char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER) as u32
}

#[derive(BitPack)]
#[bitpack(u32)]
struct Unpacking {
  /// This time we specify our handler with just a function path.
  #[bitpack(unpack_fn unpack, ..)]
  ch: u32,
}
```

```
# use bitrepr::BitPack;

#[derive(BitPack)]
#[bitpack(u32)]
struct Skip {
  /// A field may be skipped.
  ///
  /// If a field is skipped, it will not be packed into the bit representation.
  /// However, it will require that the field derives Default.
  #[bitpack(skip)]
  skip0: u32,
  /// ... or, that the field is given a defualt expression to use.
  #[bitpack(skip, default 1)]
  skip1: u32,
  /// ... or, that the field is given a defualt function to use.
  /// Is not required to be a closure.
  #[bitpack(skip, default_fn || 2)]
  skip2: u32,
}
```
