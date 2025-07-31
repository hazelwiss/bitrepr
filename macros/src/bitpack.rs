use proc_macro2::Ident;
use quote::{quote, quote_spanned};
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

fn parse(input: &DeriveInput, is_try: bool) -> syn::Result<proc_macro2::TokenStream> {
    let trait_path = if is_try {
        quote!(::bitrepr::TryBitPack)
    } else {
        quote!(::bitrepr::BitPack)
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

    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut fields = vec![];
            let mut skipped_fields = vec![];
            for (field, member) in data_struct.fields.iter().zip(data_struct.fields.members()) {
                if let Some(attr) = get_attribute(&field.attrs) {
                    let mut skip = false;

                    attr.parse_args_with(|input: ParseStream| {
                        loop {
                            if let Ok(ident) = input.parse::<syn::Ident>() {
                                if ident == "skip" {
                                    skipped_fields.push(member.clone());
                                    skip = true;
                                } else {
                                    return Err(
                                        syn::Error::new(
                                            ident.span(),
                                            "invalid identifier in attribute. \
                                            Expected either 'skip' or 'try', but got '{ident}'"
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

                                        if let Some(end) = end {
                                            if end < start {
                                                return Err(syn::Error::new(span, "range end is below range start"));
                                            }
                                        }

                                        (start, end)
                                    }
                                    _ => {
                                        return Err(syn::Error::new(span, "expected an integer or range expression"));
                                    }
                                };

                                if skip {
                                    return Err(syn::Error::new(span, "a range is given to a field with the skip attribute"));
                                }
                                if ranges.check_range(start, end) {
                                    return Err(syn::Error::new(span, "range collides with previously created range"));
                                }
                                fields.push((
                                    field.span(),
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
                                    "expected attribute arguments to contain either a bit position, bit range or 'skip', \
                                     Look at the examples showcased in the crate root documentation."
                                ));
                            }

                            return if input.is_empty() {
                                Ok(())
                            } else {
                                Err(syn::Error::new(input.span(), "invalid attribute, \
                                    expected a comma seperated list of values"))
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
            let ty_size = quote!({::core::mem::size_of::<#ty_ident>() * 8});
            let static_asserts = fields.iter().map(|(span, _, ty, start, end)| {
                let span = *span;
                let field_ty_size = quote!({::core::mem::size_of::<<#ty as #trait_path>::Repr>() * 8});
                quote_spanned!(span=>
                    const _: () = assert!(#start <= #end, "start is above or equal to end");
                    const _: () = assert!(#start < #ty_size, "start exceeds the representation type size");
                    const _: () = assert!(#end < #ty_size, "end exceeds the representation type size");
                    const _: () = assert!((#end - #start) < #field_ty_size, "the size of the bitfield is larger than the size of the representation type");
                )
            });

            let pack_fields = fields.iter().map(|(_, member, _, start, end)| {
                quote!(::bitrepr::bits::insert::<#repr_ty, _, #start, #end>(&mut #acc_ident, &#trait_path::pack(&self.#member)))
            });
            let unpack_fields = fields.iter().map(|(_, member, _, start, end)| {
                quote!(#member: #trait_path::unpack(::bitrepr::bits::extract::<#repr_ty, _, _, #start, #end>(bits)))
            }).chain(
                skipped_fields.iter().map(|member| quote!(#member: ::core::default::Default::default()))      
            );
            let unpack_fields_try = fields.iter().map(|(_, member, _, start, end)| {
                quote!(#member: #trait_path::try_unpack(::bitrepr::bits::extract::<#repr_ty, _, _, #start, #end>(bits))?)
            }).chain(
                skipped_fields.iter().map(|member| quote!(#member: ::core::default::Default::default()))      
            );

            Ok(if is_try {
                quote!(
                    #(#static_asserts)*

                    #[automatically_derived]
                    impl #trait_path for #ty_ident {
                        type Repr = #repr_ty;

                        fn pack(&self) -> #repr_ty {
                            let mut #acc_ident: #repr_ty = ::core::default::Default::default();
                            #(#pack_fields;)*
                            #acc_ident
                        }

                        fn try_unpack(bits: #repr_ty) -> Option<Self> {
                            Some(Self {
                                #(#unpack_fields_try),*
                            })
                        }
                    }
                )
            } else {
                quote!(
                    #(#static_asserts)*

                    #[automatically_derived]
                    impl #trait_path for #ty_ident {
                        type Repr = #repr_ty;

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

    parse(&input, false)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

pub(crate) fn try_bitpack2(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(ts);

    parse(&input, true)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
