use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse2, parse_quote, punctuated::Punctuated, Expr, ExprPath, Ident, Item, ItemEnum, Stmt, Token,
};

#[proc_macro_error]
#[proc_macro_derive(Protocol, attributes(protocol))]
pub fn protocol_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    protocol_derive_inner(input.into()).into()
}

fn protocol_derive_inner(input: TokenStream) -> TokenStream {
    let item = match parse2::<Item>(input) {
        Ok(syntax_tree) => syntax_tree,
        Err(error) => return error.to_compile_error(),
    };

    match item {
        // If is a struct, encode/decode all fields
        Item::Struct(protocol_struct) => {
            let protocol_struct_ident = &protocol_struct.ident;

            let mut encode_stmts: Vec<Stmt> = Vec::with_capacity(protocol_struct.fields.len());
            let mut decode_stmts: Vec<Stmt> = Vec::with_capacity(protocol_struct.fields.len());

            for field in &protocol_struct.fields {
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;

                let mut encode_ctx = quote!(());
                let mut decode_ctx = quote!(());
                let mut is_varint = false;

                for attr in &field.attrs {
                    if !attr.path().is_ident("protocol") {
                        continue;
                    }

                    let list = attr
                        .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                        .unwrap_or_else(|e| abort!(attr, e));

                    for expr in list {
                        match expr {
                            Expr::Assign(item) => match *item.left {
                                Expr::Path(ExprPath { ref path, .. }) => {
                                    match path.get_ident().unwrap().to_string().as_str() {
                                        "encode_ctx" => {
                                            encode_ctx = item.right.to_token_stream();
                                        }
                                        "decode_ctx" => {
                                            decode_ctx = item.right.to_token_stream();
                                        }
                                        "ctx" => {
                                            encode_ctx = item.right.to_token_stream();
                                            decode_ctx = encode_ctx.clone();
                                        }
                                        _ => abort!(path, "unknown option"),
                                    }
                                }
                                _ => abort!(item.left, "must be an ident"),
                            },
                            Expr::Path(ExprPath { ref path, .. }) => {
                                match path.get_ident().unwrap().to_string().as_str() {
                                    "varint" => is_varint = true,
                                    _ => abort!(path, "unknown option"),
                                }
                            }
                            _ => abort!(expr, "invalid argument"),
                        }
                    }
                }

                if is_varint {
                    encode_stmts
                        .push(parse_quote! { protocol::buf::put_varint(buf, self.#ident); });
                    decode_stmts
                        .push(parse_quote! { let #ident = protocol::buf::get_varint(buf)?; });
                } else {
                    encode_stmts.push(
                        parse_quote! { <#ty as protocol::Encodable>::encode(&self.#ident, buf, #encode_ctx)?; },
                    );
                    decode_stmts.push(
                        parse_quote! { let #ident = <#ty as protocol::Decodable>::decode(buf, #decode_ctx)?; },
                    );
                }
            }

            // Return `Self` in `decode`
            let packet_struct_field_idents: Vec<&Ident> = protocol_struct
                .fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();

            // Temporary workaround for https://github.com/dtolnay/syn/issues/1731
            match parse_quote!( Ok(Self { #(#packet_struct_field_idents),* }); ) {
                syn::Stmt::Expr(inner, _) => decode_stmts.push(syn::Stmt::Expr(inner, None)),
                _ => unreachable!(),
            }

            let out = quote! {
                impl protocol::Encodable for #protocol_struct_ident {
                    type Context = ();
                    type Error = core::convert::Infallible;

                    fn encode(&self, buf: &mut dyn bytes::BufMut, ctx: Self::Context) -> Result<(), protocol::EncodeError<Self::Error>> {
                        #(#encode_stmts)*

                        Ok(())
                    }
                }

                impl protocol::Decodable for #protocol_struct_ident {
                    type Context = ();
                    type Error = core::convert::Infallible;

                    fn decode(buf: &mut dyn bytes::Buf, ctx: Self::Context) -> Result<Self, protocol::DecodeError<Self::Error>> {
                        #(#decode_stmts)*
                    }
                }
            };
            out
        }
        // If is an enum with #[repr(i*/u*)], use FromPrimitive and ToPrimitive
        Item::Enum(protocol_enum) => {
            let ItemEnum {
                attrs: ref protocol_enum_attrs,
                ident: ref protocol_enum_ident,
                ..
            } = protocol_enum;

            let mut is_varint = false;

            for attr in protocol_enum_attrs {
                if !attr.path().is_ident("protocol") {
                    continue;
                }

                let list = attr
                    .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                    .unwrap_or_else(|e| abort!(attr, e));

                for expr in list {
                    match expr {
                        Expr::Path(ExprPath { ref path, .. }) => {
                            match path.get_ident().unwrap().to_string().as_str() {
                                "varint" => is_varint = true,
                                _ => abort!(path, "unknown option"),
                            }
                        }
                        _ => abort!(expr, "invalid argument"),
                    }
                }
            }

            let mut type_ident = None;

            for attr in protocol_enum_attrs {
                if attr.path().is_ident("repr") {
                    attr.parse_nested_meta(|meta| {
                        if [
                            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
                        ]
                        .iter()
                        .any(|t| meta.path.is_ident(t))
                        {
                            type_ident = meta.path.get_ident().cloned();
                        }

                        Ok(())
                    })
                    .unwrap();
                }
            }

            match type_ident {
                Some(type_ident) => {
                    let to_type = format_ident!("to_{}", type_ident);
                    let from_type = format_ident!("from_{}", type_ident);

                    let (encode, decode) = if is_varint {
                        if type_ident != "i32" {
                            abort!(type_ident, "varint enum must be #[repr(i32)]");
                        }

                        (
                            quote! { protocol::buf::put_varint(buf, num_traits::ToPrimitive::to_i32(self).unwrap()); },
                            quote! { Ok(num_traits::FromPrimitive::from_i32(protocol::buf::get_varint(buf)?).unwrap()) },
                        )
                    } else {
                        let put_type = format_ident!("put_{}", type_ident);
                        let get_type = format_ident!("get_{}", type_ident);

                        (
                            quote! { buf.#put_type(num_traits::ToPrimitive::#to_type(self).unwrap()); },
                            quote! { Ok(num_traits::FromPrimitive::#from_type(buf.#get_type()).unwrap()) },
                        )
                    };

                    quote! {
                        impl protocol::Encodable for #protocol_enum_ident {
                            type Context = ();
                            type Error = core::convert::Infallible;

                            fn encode(&self, buf: &mut dyn bytes::BufMut, _ctx: Self::Context) -> Result<(), protocol::EncodeError<Self::Error>> {
                                #encode

                                Ok(())
                            }
                        }

                        impl protocol::Decodable for #protocol_enum_ident {
                            type Context = ();
                            type Error = core::convert::Infallible;

                            fn decode(buf: &mut dyn bytes::Buf, _ctx: Self::Context) -> Result<Self, protocol::DecodeError<Self::Error>>
                            {
                                #decode
                            }
                        }
                    }
                }
                None => abort!(protocol_enum, "must have #[repr(i*/u*)]"),
            }
        }
        _ => abort!(item, "must be a struct"),
    }
}
