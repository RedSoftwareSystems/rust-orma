use core::ops::Deref;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::ParseStream;

use syn::parse::Parse;
use syn::*;

#[derive(Debug)]
enum AttributeType {
    Table,
    Unknown,
}

struct NamedField(Field);

impl Deref for NamedField {
    type Target = Field;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Field> for NamedField {
    fn from(field: Field) -> Self {
        Self(field)
    }
}

impl Into<Field> for NamedField {
    fn into(self) -> Field {
        self.0
    }
}

impl Parse for NamedField {
    fn parse(input: ParseStream) -> Result<Self> {
        Field::parse_named(input).map(NamedField::from)
    }
}

#[derive(Debug)]
struct DbDataAttributes {
    table: Option<String>,
}

impl From<&Ident> for AttributeType {
    fn from(ident: &Ident) -> Self {
        let str = ident.to_string();
        match str.as_str() {
            "table" => AttributeType::Table,
            _ => AttributeType::Unknown,
        }
    }
}

impl DbDataAttributes {
    fn default() -> Self {
        Self { table: None }
    }
}

fn lit_string(lit: &Lit) -> String {
    if let Lit::Str(lit_str) = lit {
        lit_str.value()
    } else {
        panic!("Invalid attr syntax {:?}", lit);
    }
}

fn parse_orma_attrs(attrs: &[NestedMeta]) -> DbDataAttributes {
    let mut ctx = DbDataAttributes::default();
    for attr in attrs {
        let meta = if let NestedMeta::Meta(meta) = attr {
            meta
        } else {
            panic!("{:?} is not a Meta", attr);
        };
        if let Meta::NameValue(name_value) = meta {
            let attr_type = AttributeType::from(name_value.path.get_ident().unwrap());
            if let AttributeType::Table = attr_type {
                ctx.table = Some(lit_string(&name_value.lit))
            }
        };
    }
    ctx
}

pub fn impl_orma(attrs: &[NestedMeta], input: &mut DeriveInput) -> TokenStream {
    let dbdata_attrs = parse_orma_attrs(attrs);
    let table_name = if let Some(table_name) = dbdata_attrs.table {
        table_name
    } else {
        panic!("No table name provided, {:?}", attrs);
    };
    let data = if let Data::Struct(ref mut it) = input.data {
        it
    } else {
        return Error::new(input.ident.span(), "Only structs are supported").to_compile_error();
    };

    let fields = if let Fields::Named(ref mut it) = data.fields {
        it
    } else {
        return Error::new(input.ident.span(), "Tuple structs are not supported")
            .to_compile_error();
    };

    let skip: Attribute = parse_quote! {#[serde(skip)]};
    fields.named.push(Field {
        attrs: vec![skip],
        vis: parse_quote! {pub},
        ident: Some(Ident::new("orma_id", Span::call_site())),
        colon_token: None,
        ty: parse_quote! {Option<::uuid::Uuid>},
    });

    let skip: Attribute = parse_quote! {#[serde(skip)]};
    fields.named.push(Field {
        attrs: vec![skip],
        vis: parse_quote! {pub},
        ident: Some(Ident::new("orma_version", Span::call_site())),
        colon_token: None,
        ty: parse_quote! {Option<i32>},
    });

    let ident = &input.ident;
    quote! {
        #input
        impl ::orma::DbData for #ident {
            fn table_name() -> &'static str {
                #table_name
            }
            fn id(&self) -> Option<::uuid::Uuid> {self.orma_id}
            fn version(&self) -> Option<i32> {self.orma_version}
            fn set_id(&mut self, uuid: ::uuid::Uuid) {
                self.orma_id = Some(uuid);
            }
            fn set_version(&mut self, version: i32) {
                self.orma_version = Some(version);
            }

        }
    }
}
