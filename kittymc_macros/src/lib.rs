use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(SerializePacketFunc)]
pub fn derive_serialize_packet_func(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;

    let data_enum = match input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                name,
                "SerializePacketFunc can only be derived for enums"
            )
                .to_compile_error()
                .into();
        }
    };

    let variant_arms = data_enum.variants.iter().map(|variant| {
        let vname = &variant.ident;
        quote! {
            Self::#vname(inner) => inner.serialize(),
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn serialize(&self) -> Vec<u8> {
                match self {
                    #(#variant_arms)*
                }
            }
        }
    };

    expanded.into()
}
