use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Field};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let DeriveInput {
        ref ident,
        ref data,
        ..
    } = input;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = data
    {
        &fields.named
    } else {
        unimplemented!()
    };

    let field_is_optional = |field: &Field| -> bool {
        if let syn::Type::Path(ref path) = field.ty {
            path.path.segments.last().unwrap().ident == "Option"
        } else {
            false
        }
    };

    let last_ident = |ty: &syn::Type| -> syn::Ident {
        if let syn::Type::Path(ref path) = ty {
            path.path.segments.last().as_ref().unwrap().ident.clone()
        } else {
            unimplemented!()
        }
    };

    let type_inside_option = |ty: &syn::Type| -> syn::Ident {
        if let syn::Type::Path(ref path) = ty {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                ref args,
                ..
            }) = path.path.segments.last().unwrap().arguments
            {
                if let syn::GenericArgument::Type(ref ty) = args[0] {
                    last_ident(ty)
                } else {
                    unimplemented!()
                }
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    };

    // thing: Option<Thing>
    let fields_by_name_option_type = fields.iter().map(|field @ Field { ident, ty, .. }| {
        if field_is_optional(field) {
            quote! { #ident:  #ty /*optional field*/}
        } else {
            quote! { #ident:  Option<#ty> /*required field*/ }
        }
    });

    // fn thing(&mut self, thing: Thing) -> Struct
    let fields_build_methods = fields.iter().map(|field @ Field { ident, ty, .. }| {
        if field_is_optional(field) {
            let unoptional = type_inside_option(ty);
            quote! {
                pub fn #ident(&mut self, #ident: #unoptional) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        } else {
            quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        }
    });

    // thing: self.thing.clone().ok_or("no such Thing")?;
    let fields_getters = fields.iter().map(|field @ Field { ident, .. }| {
        if field_is_optional(field) {
            quote! {
                #ident: self.#ident.clone(),
            }
        } else {
            quote! {
                #ident: self.#ident.clone().ok_or(concat!("the value of ", stringify!(#ident), "was not set"))?,
            }}
    });

    // let builder_ident = syn::Ident::new(&(ident.to_string() + "Builder"), ident.span());
    let builder_ident = format_ident!("{}Builder", ident);
    let expanded = quote! {
        #[derive(Default)]
        pub struct #builder_ident{
            #(#fields_by_name_option_type),*
        }

        impl #builder_ident {
            #(#fields_build_methods)*

            pub fn build(&self) -> Result<#ident, Box<dyn std::error::Error>> {
                Ok(#ident {
                    #(#fields_getters)*
                })
            }
        }

        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident ::default()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
