use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse2, Item};

#[proc_macro_error]
#[proc_macro_derive(Packet)]
pub fn packet_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    packet_derive_inner_new(input.into()).into()
}

fn packet_derive_inner_new(input: TokenStream) -> TokenStream {
    let item = match parse2::<Item>(input) {
        Ok(syntax_tree) => syntax_tree,
        Err(error) => return error.to_compile_error(),
    };

    match item {
        Item::Enum(packet_enum) => {
            let packet_enum_ident = &packet_enum.ident;
            let packet_enum_variants = packet_enum
                .variants
                .iter()
                .map(|v| &v.ident)
                .collect::<Vec<_>>();
            let packet_enum_discriminants = packet_enum
                .variants
                .iter()
                .map(|v| &v.discriminant.as_ref().unwrap().1)
                .collect::<Vec<_>>();

            quote! {
                impl protocol::Encodable for #packet_enum_ident {
                    type Context = ();
                    type Error = core::convert::Infallible;

                    fn encode(
                        &self,
                        buf: &mut dyn BufMut,
                        _ctx: Self::Context,
                    ) -> Result<(), protocol::EncodeError<Self::Error>> {
                        match self {
                            #(Self::#packet_enum_variants(packet) => packet.encode(buf, ()),)*
                        }
                    }
                }

                impl protocol::Decodable for #packet_enum_ident {
                    type Context = packet::PacketDecodeContext;
                    type Error = packet::PacketDecodeError;

                    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, protocol::DecodeError<Self::Error>>
                    where
                        Self: Sized,
                    {
                        Ok(match ctx.packet_id {
                            #(#packet_enum_discriminants => Self::#packet_enum_variants(protocol::buf::ResultExpandExt::expand(#packet_enum_variants::decode(buf, ()))?),)*
                            id => return Err(protocol::DecodeError::Other(packet::PacketDecodeError::InvalidPacketId(id))),
                        })
                    }
                }

                impl packet::Packet for #packet_enum_ident {
                    fn get_id(&self) -> i32 {
                        match self {
                            #(Self::#packet_enum_variants(packet) => packet.get_id(),)*
                        }
                    }
                }
            }
        }
        _ => abort!(item, "must be an enum of types implementing Packet"),
    }
}
