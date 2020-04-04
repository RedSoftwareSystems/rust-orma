#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RedSoftwareSystems/rust-orma/master/orma.svg?sanitize=true"
)]
//! This crate provides derive macros for orma.
//!
//! ## Example
//!
//! ```edition2018
//!  use orma::*;
//!  use serde::{Serialize, Deserialize};
//!
//!  #[orma_derive::orma_obj(table = "table_name")]
//!  #[derive(Serialize, Deserialize)]
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
mod test_impl;

use syn::{parse_macro_input, AttributeArgs, DeriveInput};

/// This crate provides derive macros for orma.
///
/// ## Example
///
/// ```edition2018
///  use serde::{Serialize, Deserialize};
///
///  #[orma_derive::orma_obj(table = "table_name")]
///  #[derive(Serialize, Deserialize)]
///  struct TestData {
///      field_1: String,
///      field_2: String,
///      some_other_filed: String,
///  }
///
/// ```
///
#[proc_macro_attribute]
pub fn orma_obj(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse tokens
    let args = parse_macro_input!(args as AttributeArgs);
    let mut input: DeriveInput = parse_macro_input!(input);
    let gen = dbdata::impl_orma(&args, &mut input);

    // println!(
    //     r#"
    // --------------------- RESULT CODE ---------------------
    // {}
    // -------------------------------------------------------"#,
    //     gen
    // );
    // Return the generated impl
    gen.into()
}

/// This macro produces an async test.
///
/// For each argument there is a function with the same name that provides the value for that argument and receives as input
/// the name of the test function (as &str)
///
/// # Example:
/// ```edition2018
/// fn connection(input: &str) -> &str {
///     input
/// }
///
/// #[orma_derive::test]
/// async fn test_orma_test(connection: &str) {
///     assert_eq!(data, "test_orma_test");
/// }
/// ```
#[proc_macro_attribute]
pub fn test(
    _args: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse tokens
    let ast = syn::parse(body).unwrap();

    // Build impl
    let gen = test_impl::test_impl(ast);

    // println!(
    //     r#"
    // --------------------- RESULT CODE ---------------------
    // {}
    // -------------------------------------------------------"#,
    //     gen
    // );
    // Return the generated impl

    gen.into()
}
