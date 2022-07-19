use std::collections::HashMap;

use quote::quote;
use syn::{parse::Parse, parse_macro_input, Expr, ExprLit, Ident, ItemStruct, Lit, LitBool, Token};

struct Field {
    ident: Ident,
    _equal_token: Token![=],
    value: Expr,
}
impl Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Field {
            ident: input.parse()?,
            _equal_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct Fields(HashMap<String, Expr>);
impl Parse for Fields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Fields(
            input
                .parse_terminated::<_, Token![,]>(Field::parse)?
                .into_iter()
                .map(|f| (f.ident.to_string(), f.value))
                .collect(),
        ))
    }
}

#[proc_macro_attribute]
pub fn node_type(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let Fields(attrs) = parse_macro_input!(attr as Fields);
    let name = attrs.get("name").unwrap();
    let category = attrs.get("category").unwrap();
    let children = attrs
        .get("children")
        .map(|expr| match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, .. }),
                ..
            }) => *value,
            _ => panic!("unsupported value for children"),
        })
        .unwrap_or(false);

    let item = parse_macro_input!(item as ItemStruct);
    let item_name = item.ident.clone();

    let fields: Vec<_> = item
        .fields
        .iter()
        .map(|f| {
            let field_attr = f.attrs.iter().find(|a| a.path.is_ident("field")).unwrap();
            let attr_fields: Fields = field_attr.parse_args().unwrap();
            (
                f.ident.as_ref().unwrap().clone(),
                f.ty.clone(),
                attr_fields.0.clone(),
            )
        })
        .collect();

    let struct_fields = fields
        .iter()
        .map(|(ident, ty, _)| quote! { pub #ident: #ty });

    let struct_inits = fields
        .iter()
        .map(|(ident, _, attrs)| {
            (
                ident,
                attrs
                    .get("default")
                    .expect("expected default field in attribute"),
            )
        })
        .map(|(ident, init)| quote! { #ident: #init });

    let item_diff_name = quote::format_ident!("{}Diff", item_name);
    let struct_diff_fields = fields
        .iter()
        .map(|(ident, ty, _)| quote! { pub #ident: Option<#ty> });
    let change_field_checks = fields
        .iter()
        .map(|(ident, _, _)| quote! { self.#ident.is_some() });
    let apply_stmts = fields
        .iter()
        .map(|(ident, _, _)| quote! { self.#ident = diff.#ident.unwrap_or(self.#ident) });

    let ts = quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct #item_name {
            #(#struct_fields),*
        }
        impl #item_name {
            pub const fn new() -> Self {
                Self {
                    #(#struct_inits),*
                }
            }
        }
        impl Default for #item_name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl NodeDataMeta for #item_name {
            fn name(&self) -> &'static str {
                #name
            }
            fn category(&self) -> NodeCategory {
                #category
            }
            fn can_have_children(&self) -> bool {
                #children
            }
        }
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct #item_diff_name {
            #(#struct_diff_fields),*
        }
        impl #item_diff_name {
            pub fn into_option(self) -> Option<Self> {
                let has_changes = #(#change_field_checks)||*;
                has_changes.then_some(self)
            }
        }
        impl #item_name {
            pub fn apply(&mut self, diff: #item_diff_name) {
                #(#apply_stmts);*;
            }
        }
    };

    ts.into()
}
