use proc_macro2::Ident;
use quote::{quote, quote_spanned, ToTokens};
use syn::{DeriveInput, parse::ParseStream, spanned::Spanned};

const ATTR_STR: &str = "bitpack";

fn get_attribute(attributes: &[syn::Attribute]) -> Option<&syn::Attribute> {
    attributes
        .iter()
        .find(|&attr| attr.path().is_ident(ATTR_STR))
}

fn parse_as_unsigned_int(expr: &syn::Expr) -> syn::Result<usize> {
    if let syn::Expr::Lit(lit) = expr {
        if let syn::Lit::Int(int) = &lit.lit {
            int.base10_parse()
        } else {
            Err(syn::Error::new(lit.span(), "expected an integer literal"))
        }
    } else {
        Err(syn::Error::new(
            expr.span(),
            "expected a literal expression",
        ))
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Unsafe,
    Try,
    Normal,
}

impl Kind {
    fn is_try(self) -> bool {
        matches!(self, Self::Try)
    }

    fn is_unsafe(self) -> bool {
        matches!(self, Self::Unsafe)
    }
}

fn parse(input: &DeriveInput, kind: Kind) -> syn::Result<proc_macro2::TokenStream> {
    let repr_trait = quote!(::bitrepr::BitRepr);
    let unsafe_trait = quote!(::bitrepr::UnsafeBitPack);
    let try_trait =  quote!(::bitrepr::TryBitPack);
    let trait_ = quote!(::bitrepr::BitPack);
    let self_trait = if kind.is_unsafe() {
        unsafe_trait.clone()
    } else if kind.is_try() {
        try_trait.clone()
    } else {
        trait_.clone()
    };

    let mut repr_ty: Option<syn::Path> = None;
    if let Some(outer_attr) = get_attribute(&input.attrs) {
        outer_attr.parse_nested_meta(|meta| {
            repr_ty = Some(meta.path);
            Ok(())
        })?;
    }

    let Some(repr_ty) = repr_ty else {
        return Err(syn::Error::new(
            input.span(),
            "missing #[bitpack(ty)] attribute at the top denoting represented type",
        ));
    };

    struct Ranges(Vec<(usize, Option<usize>)>);

    impl Ranges {
        fn check_range(&mut self, start: usize, end: Option<usize>) -> bool {
            let overlap = self.overlap(start, end);
            self.0.push((start, end));
            overlap
        }

        fn overlap(&self, start: usize, end: Option<usize>) -> bool {
            self.0.iter().any(|(s, e)| {
                let hit = |p0: usize, p1: Option<usize>, s: usize, e: Option<usize>| -> bool {
                    if let Some(e) = e {
                        let range = s..=e;
                        if let Some(p1) = p1 {
                            range.contains(&p0) || range.contains(&p1)
                        } else {
                            range.contains(&p0)
                        }
                    } else if let Some(p1) = p1 {
                        p0 >= s || p1 >= s
                    } else {
                        true
                    }
                };

                hit(start, end, *s, *e) || hit(*s, *e, start, end)
            })
        }
    }

    let mut ranges = Ranges(vec![]);

    #[derive(Default)]
    struct FieldAttributes {
        unwrap: bool,
        unsafe_: bool,
        pack_fn: Option<syn::Expr>,
        unpack_fn: Option<syn::Expr>,
    }

    #[derive(Default)]
    struct SkippedFieldAttributes {
        default: Option<syn::Expr>,
        default_fn: Option<syn::Expr>,
    }

    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut fields = vec![];
            let mut skipped_fields = vec![];
            for (field, member) in data_struct.fields.iter().zip(data_struct.fields.members()) {
                if let Some(attr) = get_attribute(&field.attrs) {
                    let mut skip = false;
                    let mut fattrs = FieldAttributes::default();
                    let mut skip_fattrs =SkippedFieldAttributes::default();
                    let mut added_field = false;

                    attr.parse_args_with(|input: ParseStream| {
                        loop {
                            if input.parse::<syn::Token![unsafe]>().is_ok() {
                                fattrs.unsafe_ = true;
                                if input.parse::<syn::Token![,]>().is_ok() {
                                    continue;
                                }
                            } else if let Ok(ident) = input.parse::<syn::Ident>() {
                                if ident == "skip" {
                                    skip = true;
                                } else if ident == "unwrap" {
                                    fattrs.unwrap = true; 
                                } else if ident == "pack_fn" {
                                    fattrs.pack_fn = Some(input.parse()?);
                                } else if ident == "unpack_fn" {
                                    fattrs.unpack_fn = Some(input.parse()?);
                                } else if ident == "default" {
                                    skip_fattrs.default = Some(input.parse()?);
                                } else if ident == "default_fn" {
                                    skip_fattrs.default_fn = Some(input.parse()?);
                                } else {
                                    return Err(
                                        syn::Error::new(
                                            ident.span(),
                                            "invalid identifier in attribute. \
                                            Expected either 'skip', 'unwrap', 'unsafe', 'pack_fn', 'unpack_fn' \
                                            'default' or 'default_fn' but got '{ident}'"
                                        )
                                    );
                                }
                                if input.parse::<syn::Token![,]>().is_ok() {
                                    continue;
                                }
                            } else if let Ok(expr) = input.parse::<syn::Expr>() {
                                let span = expr.span();
                                let (start, end) = match expr {
                                    syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Int(pos),
                                        ..
                                    }) => {
                                        let pos: usize = pos.base10_parse()?;
                                        (pos, Some(pos))
                                    }
                                    syn::Expr::Range(syn::ExprRange {
                                        start, end, limits, ..
                                    }) => {
                                        let start = if let Some(start) = start {
                                            parse_as_unsigned_int(&start)?
                                        } else {
                                            0
                                        };

                                        let end = if let Some(end) = end {
                                            Some(parse_as_unsigned_int(&end)?)
                                        } else {
                                            // 
                                            None
                                        };

                                        let end = match limits {
                                            syn::RangeLimits::Closed(..) => end,
                                            syn::RangeLimits::HalfOpen(..) => if let Some(end) = end {
                                                if end > 0 {
                                                    Some(end - 1)
                                                } else {
                                                    return Err(syn::Error::new(span, "range end wraps from zero. Did you mean to use inclusive range?"));
                                                }
                                            } else {
                                                None
                                            },
                                        };

                                        if let Some(end) = end && end < start {
                                            return Err(syn::Error::new(span, "range end is below range start"));
                                        }

                                        (start, end)
                                    }
                                    _ => {
                                        return Err(syn::Error::new(span, "expected an integer or range expression"));
                                    }
                                };

                                if skip {
                                    return Err(syn::Error::new(attr.span(), "a range is given to a field with the skip attribute"));
                                }
                                if skip_fattrs.default.is_some() || skip_fattrs.default_fn.is_some() {
                                    return Err(syn::Error::new(attr.span(), "default values cannot be specified in a field without `skip`"));
                                }
                                if ranges.check_range(start, end) {
                                    return Err(syn::Error::new(span, "range collides with previously created range"));
                                }
                                if fattrs.unwrap && fattrs.unsafe_ {
                                    return Err(syn::Error::new(attr.span(), "unwrap and unsafe properties are mutually exclusive"));
                                }

                                added_field |= true;
                                fields.push((
                                    field.span(),
                                    fattrs,
                                    member.clone(),
                                    &field.ty,
                                    quote!(#start),
                                    end.map_or_else(
                                        || quote!({ ::core::mem::size_of::<#repr_ty>() * 8 - 1 }),
                                        |v| quote!(#v) )
                                    )
                                );
                            } else {
                                return Err(syn::Error::new(
                                    attr.span(),
                                    "expected valid attribute. Look in the crate root documentation for more details."
                                ));
                            }

                            if skip {
                                if skip_fattrs.default.is_some() && skip_fattrs.default_fn.is_some() {
                                    return Err(syn::Error::new(attr.span(), "cannot specify a default expression and function at the same time"));
                                }
                                skipped_fields.push((skip_fattrs, member.clone()));
                            }

                            return if input.is_empty() && (skip  || added_field ) {
                                Ok(())
                            } else {
                                Err(syn::Error::new(input.span(), "invalid attribute, \
                                    expected a comma seperated list of values. See crate \
                                    root documenation for more details on correct attribute \
                                    syntax"))
                            }
                        }
                    })?;
                } else {
                    return Err(syn::Error::new(
                        field.span(),
                        "field missing bitpack attribute",
                    ));
                }
            }

            let acc_ident = Ident::new("repr", proc_macro2::Span::call_site());
            let ty_ident = &input.ident;
            let static_asserts = fields.iter().map(|(span, _, _, ty, start, end)| {
                let span = *span;
                let ty_size = quote!({::core::mem::size_of::<<#ty_ident as #repr_trait>::Repr>() * 8});
                let field_ty_size = quote!({::core::mem::size_of::<<#ty as #repr_trait>::Repr>() * 8});
                quote_spanned!(span=>
                    const _: () = assert!(#start <= #end, "start is above or equal to end");
                    const _: () = assert!(#start < #ty_size, "start exceeds the representation type size");
                    const _: () = assert!(#end < #ty_size, "end exceeds the representation type size");
                    const _: () = assert!((#end - #start) < #field_ty_size, "the size of the bitfield is larger than the size of the representation type");
                )
            });

            let pack_fields = fields.iter().map(|(_, attrs, member, ty, start, end)| {
                let f = if attrs.unwrap || kind.is_try() {
                    quote!(<#ty as #try_trait>::pack)
                } else if attrs.unsafe_ || kind.is_unsafe() {
                    quote!(<#ty as #unsafe_trait>::pack)
                } else if let Some(pack_fn) = &attrs.pack_fn {
                    quote!(#pack_fn)
                } else {
                    quote!(<#ty as #self_trait>::pack)
                };

                quote!(
                    {
                        let f: fn(&#ty) -> <#ty as #repr_trait>::Repr = #f;
                        ::bitrepr::bits::insert::<#repr_ty, _, #start, #end>(&mut #acc_ident, &f(&self.#member))
                    }
                )
            });

            let unpack_fields = fields.iter().map(|(_, attrs, member, ty, start, end)| {
                let mut unsafe_kw = None;
                let mut try_ = None;
                let mut unwrap = None;
                let mut f;
                let f_ty;
                if attrs.unwrap {
                    f = quote!(<#ty as #try_trait>::try_unpack);
                    f_ty = quote!(fn(<#ty as #repr_trait>::Repr) -> Option<#ty>);
                    unwrap = Some(quote!(.unwrap()));
                } else if attrs.unsafe_ {
                    f = quote!(<#ty as #unsafe_trait>::unsafe_unpack);
                    f_ty = quote!(unsafe fn(<#ty as #repr_trait>::Repr) -> #ty);
                    unsafe_kw = Some(quote!(unsafe));
                } else if kind.is_unsafe() {
                    f = quote!(<#ty as #unsafe_trait>::unsafe_unpack);
                    f_ty = quote!(unsafe fn(<#ty as #repr_trait>::Repr) -> #ty);
                } else if kind.is_try() {
                    f = quote!(<#ty as #try_trait>::try_unpack);
                    f_ty = quote!(fn(<#ty as #repr_trait>::Repr) -> Option<#ty>);
                    try_ = Some(quote!(?));
                } else {
                    f = quote!(<#ty as #trait_>::unpack);
                    f_ty = quote!(fn(<#ty as #repr_trait>::Repr) -> #ty);
                }

                if let Some(f_) = &attrs.unpack_fn {
                    f = f_.to_token_stream();
                }

                quote!(
                    #member: #unsafe_kw {
                        let f: #f_ty = #f;
                        f(::bitrepr::bits::extract::<#repr_ty, _, _, #start, #end>(bits)) #try_ #unwrap
                    }
                )
            }).chain(
                skipped_fields.iter().map(|(attr, member)| {
                    let f = if let Some(d) = &attr.default {
                        d.to_token_stream()
                    } else if let Some(d) = &attr.default_fn {
                        quote!((#d)())
                    } else {
                        quote!(::core::default::Default::default())
                    };
                    quote!(#member: #f)
                })
            );

            let impl_bitrepr = quote!(
                #[automatically_derived]
                impl #repr_trait for #ty_ident {
                    type Repr = #repr_ty;
                }
            );

            Ok(if kind.is_unsafe() {
                quote!(
                    #(#static_asserts)*
                    #impl_bitrepr

                    #[automatically_derived]
                    impl #self_trait for #ty_ident {
                        fn pack(&self) -> #repr_ty {
                            let mut #acc_ident: #repr_ty = ::core::default::Default::default();
                            #(#pack_fields;)*
                            #acc_ident
                        }

                        unsafe fn unsafe_unpack(bits: #repr_ty) -> Self {
                            Self {
                                #(#unpack_fields),*
                            }
                        }
                    }
                )
            } else if kind.is_try() {
                quote!(
                    #(#static_asserts)*
                    #impl_bitrepr

                    #[automatically_derived]
                    impl #self_trait for #ty_ident {
                        fn pack(&self) -> #repr_ty {
                            let mut #acc_ident: #repr_ty = ::core::default::Default::default();
                            #(#pack_fields;)*
                            #acc_ident
                        }

                        fn try_unpack(bits: #repr_ty) -> Option<Self> {
                            Some(Self {
                                #(#unpack_fields),*
                            })
                        }
                    }
                )
            } else {
                quote!(
                    #(#static_asserts)*
                    #impl_bitrepr

                    #[automatically_derived]
                    impl #self_trait for #ty_ident {
                        fn pack(&self) -> #repr_ty {
                            let mut #acc_ident: #repr_ty = ::core::default::Default::default();
                            #(#pack_fields;)*
                            #acc_ident
                        }

                        fn unpack(bits: #repr_ty) -> Self {
                            Self {
                                #(#unpack_fields),*
                            }
                        }
                    }
                )
            })
        }
        syn::Data::Enum(_) => Err(syn::Error::new(
            input.span(),
            "enums are currently unsupported for bitpacking",
        )),
        syn::Data::Union(_) => Err(syn::Error::new(
            input.span(),
            "unions are currently unsupported for bitpacking",
        )),
    }
}

pub(crate) fn bitpack2(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(ts);

    parse(&input, Kind::Normal)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

pub(crate) fn try_bitpack2(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(ts);

    parse(&input, Kind::Try)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

pub(crate) fn unsafe_bitpack2(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(ts);

    parse(&input, Kind::Unsafe)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
