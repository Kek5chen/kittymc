use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(PacketHelperFuncs)]
pub fn derive_packet_helper_funcs(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;

    let data_enum = match input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                name,
                "SerializePacketFunc can only be derived for enums"
            ).to_compile_error().into();
        }
    };

    let variant_arms = data_enum.variants.iter().map(|variant| {
        let vname = &variant.ident;
        let unnamed_fields = match &variant.fields {
            Fields::Unnamed(fields) if !fields.unnamed.is_empty() => fields,
            _ => {
                return Err(syn::Error::new_spanned(
                    vname,
                    "Packet Enum Variant MUST have an inner struct that does serialization and deserialization."
                ).to_compile_error().into());
            }
        };

        let inner_field_ty = (&unnamed_fields.unnamed[0]).ty.clone();

        Ok((quote! {
            Self::#vname(inner) => inner.serialize(),
        },
        quote! {
            Self::#vname(_) => #inner_field_ty::name(),
        }))
    });

    let results: Vec<Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), proc_macro2::TokenStream>> = variant_arms.clone().collect();

    if results.iter().any(|v| v.is_err()) {
        let mut error_collector = proc_macro2::TokenStream::new();
        error_collector.append_all(
            results
                .into_iter()
                .filter(|res| res
                    .is_err())
                .map(|res| res.unwrap_err()));
        return error_collector.into();
    }

    let results = results
        .into_iter()
        .map(|res| res.unwrap());

    let mut serializers = vec![];
    let mut names = vec![];

    for (s, n) in results {
        serializers.push(s);
        names.push(n);
    }

    let expanded = quote! {
        impl #name {
            pub fn serialize(&self) -> Vec<u8> {
                match self {
                    #(#serializers)*
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    #(#names)*
                }
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Packet)]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;
    let generics = input.generics.params;

    let expanded = quote! {
        impl<#generics> crate::packets::packet_serialization::NamedPacket for #name<#generics> {
            fn name() -> &'static str {
                stringify!(#name)
            }
        }
    };

    expanded.into()
}
