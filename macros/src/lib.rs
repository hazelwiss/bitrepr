mod bitpack;

#[proc_macro_derive(TryBitPack, attributes(bitpack))]
pub fn try_bitpack(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bitpack::try_bitpack2(ts)
}

#[proc_macro_derive(BitPack, attributes(bitpack))]
pub fn bitpack(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bitpack::bitpack2(ts)
}
