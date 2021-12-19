use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: name, data, ..
    } = parse_macro_input!(input as DeriveInput);
    eprintln!("{:#?}", data);

    let fields_stream = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let collected = fields.named.iter().map(|field| {
                let name = &field.ident;
                quote_spanned! {field.span() =>
                    .field(stringify!(#name), &self.#name)
                }
            });
            quote! {
                #(#collected)*
            }
        }
        _ => todo!(),
    };

    let expanded = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                #fields_stream
                .finish()
            }
        }
    };
    TokenStream::from(expanded)
}
