#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RedSoftwareSystems/rust-orma/master/orma.svg?sanitize=true"
)]
//! This crate provides Orma's derive macros.
//!
//! # Example
//!
//! ```edition2018
//!  use orma::*;
//!  use orma_derive::DbData;
//!  use serde_derive::Serialize;
//!
//!  #[pk(field_1 = "field1", field_2 = "field2")]
//!  #[table("table_name")]
//!  #[derive(Serialize, DbData)]
//!  struct TestData {
//!      field_1: String,
//!      field_2: String,
//!      some_other_filed: String,
//!  }
//!
//! ```
//!
extern crate proc_macro;

mod dbdata;

use proc_macro::TokenStream;

use syn;

#[proc_macro_derive(DbData, attributes(schema, table, pk))]
pub fn dbdata_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    dbdata::impl_dbdata_macro(&ast).into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
