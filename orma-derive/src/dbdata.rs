use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::{DeriveInput, Lit, Meta, MetaList, NestedMeta};

enum AttributeType {
    Table,
    Pk,
    Unknown,
}

struct PkItem {
    column_name: Literal,
    field_name: Ident,
}

impl ToTokens for PkItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (col_name, field_name) = (&self.column_name, &self.field_name);
        tokens.extend(quote! { (#col_name, &self.#field_name) });
    }
}

struct DbDataAttributes {
    table: Option<String>,
    pk: Option<Vec<PkItem>>,
}

impl From<&Ident> for AttributeType {
    fn from(ident: &Ident) -> Self {
        let str = format!("{}", ident);
        match str.as_str() {
            "orma_table" => AttributeType::Table,
            "orma_pk" => AttributeType::Pk,
            _ => AttributeType::Unknown,
        }
    }
}

pub fn impl_dbdata_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let attrs = field_attrs(ast);
    let table = attrs.table.unwrap_or_else(|| format!("{}", name));
    let pk = attrs.pk.unwrap_or_else(|| vec![]);

    let gen = quote! {
        impl orma::DbData for #name {
            fn table_name() -> &'static str {
                #table
            }
            fn pk_filter(&self) -> Vec<(&str, &(dyn orma::ToSql + Sync))> {
                vec![#(#pk),*]
            }
        }
    };

    gen
}

impl DbDataAttributes {
    fn default() -> Self {
        Self {
            table: None,
            pk: None,
        }
    }
}

fn attribute_string_val(meta_list: &MetaList) -> String {
    let name_token = meta_list.nested.first().unwrap();
    match name_token {
        NestedMeta::Lit(lit) => {
            if let Lit::Str(lit_str) = lit {
                lit_str.value()
            } else {
                panic!("table attribute should be a string")
            }
        }
        _ => panic!("no value defined for macro attr"),
    }
}

fn lit_string(lit: &Lit) -> String {
    if let Lit::Str(lit_str) = lit {
        lit_str.value()
    } else {
        panic!("Invalid attr syntax {:?}", lit)
    }
}

fn kv_pk(kv_nested_meta: &NestedMeta) -> PkItem {
    match kv_nested_meta {
        NestedMeta::Meta(meta) => match meta {
            Meta::NameValue(name_value) => PkItem {
                field_name: name_value.path.get_ident().unwrap().clone(),
                column_name: Literal::string(&lit_string(&name_value.lit)),
            },
            _ => panic!("Invalid attr syntax {:?}", kv_nested_meta),
        },
        _ => panic!("Invalid attr syntax {:?}", kv_nested_meta),
    }
}

fn attribute_pk(meta_list: &MetaList) -> Vec<PkItem> {
    meta_list
        .nested
        .iter()
        .map(|kv_nested_meta| kv_pk(kv_nested_meta))
        .collect()
}

fn field_attrs(ast: &DeriveInput) -> DbDataAttributes {
    let mut ctx = DbDataAttributes::default();
    for attr in &ast.attrs {
        let attr: Meta = attr.parse_meta().unwrap();
        if let Meta::List(meta_list) = attr {
            let attr_type = AttributeType::from(meta_list.path.get_ident().unwrap());
            match attr_type {
                AttributeType::Table => ctx.table = Some(attribute_string_val(&meta_list)),
                AttributeType::Pk => ctx.pk = Some(attribute_pk(&meta_list)),
                _ => (),
            }
        };
    }
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use std::str::FromStr;
    #[test]
    fn test_pk_to_token() {
        let pk = PkItem {
            column_name: Literal::string("column_name"),
            field_name: Ident::new("field_name", Span::call_site()),
        };

        let expected = TokenStream::from_str("(\"column_name\", &self.field_name)").unwrap();

        assert_eq!(format!("{}", expected), format!("{}", pk.to_token_stream()));
    }
}
