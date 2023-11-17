use proc_macro_error::abort;
use syn::{
    punctuated::Punctuated, spanned::Spanned, DeriveInput, ExprLit, ExprTuple, Ident, Lit,
    MetaNameValue, Token,
};

#[proc_macro_derive(ImageBank, attributes(image, scale))]
pub fn image_bank(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct ImageBankMeta<'a> {
        field_id: &'a Ident,
        path: String,
    }
    let input: DeriveInput = syn::parse(input).expect("invalid input for image bank");
    let id = &input.ident;
    let syn::Data::Struct(data) = input.data else {
        abort!(input.span(), "image bank works only on structs")
    };

    let mut meta = vec![];

    for field in data.fields.iter() {
        for attr in field.attrs.iter() {
            if attr.meta.path().is_ident("image") {
                let name = field.ident.as_ref().unwrap();
                let mut bank = ImageBankMeta {
                    field_id: name,
                    path: String::new(),
                };

                let pairs = attr.meta.require_list().expect("must be list");
                let punct = pairs
                    .parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)
                    .expect("must be name value pairs");

                for item in punct {
                    if item.path.is_ident("path") {
                        let syn::Expr::Lit(ExprLit { ref lit, .. }) = item.value else {
                            abort!(item.span(), "path must be str lit")
                        };
                        let Lit::Str(str) = lit else {
                            abort!(lit.span(), "must be str lit")
                        };
                        bank.path = format!("/{}", str.value());
                    }

                    if item.path.is_ident("scale") {
                        let syn::Expr::Tuple(ExprTuple { ref elems, .. }) = item.value else {
                            abort!(item.span(), "path must be str lit")
                        };
                    }
                }
                meta.push(bank);
            }
        }
    }

    let tokens = meta.iter().map(|bank| {
        let field = &bank.field_id;
        let img_path = &bank.path;
        quote::quote!(#field: ggez::graphics::Image::from_path(ctx, #img_path)?,)
    });

    quote::quote!(
        impl #id {
            pub fn new(ctx: &mut ggez::Context) -> Result<Self, ggez::GameError> {
                Ok(Self {
                    #(#tokens)*
                })
            }
        }
    )
    .into()
}
