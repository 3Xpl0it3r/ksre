use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(ToBytes)]
pub fn bytes_serializer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let compile_err = TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only struct with named fields can derive `FromRwo`",
        )
        .to_compile_error(),
    );
    let name = input.ident;

    match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let field_vals_encode = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        buffer.extend(self.#name.byte_encode());
                    }
                });
                let field_vals_decode = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        let readed = self.#name.byte_decode(&buffer[offset..]);
                        offset += readed;
                    }
                });
                TokenStream::from(quote! {
                    impl #name {
                        fn serialize(&self) -> Vec<u8> {
                            let mut buffer = Vec::new();
                            #(#field_vals_encode;)*
                            buffer
                        }

                        fn deserialize(&mut self, buffer: &[u8])->usize {
                            let mut offset = 0;
                            #(#field_vals_decode;)*
                            offset
                        }
                    }
                })
            }
            syn::Fields::Unnamed(_) => compile_err,
            syn::Fields::Unit => compile_err,
        },
        syn::Data::Enum(_) => compile_err,
        syn::Data::Union(_) => compile_err,
    }
}
