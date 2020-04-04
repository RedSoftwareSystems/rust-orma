#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RedSoftwareSystems/rust-orma/master/orma.svg?sanitize=true"
)]
//! # orma
//!
//! A PostgreSQL ORM written in Rust language
//!
//! ## Introduction
//!
//! When you feel the need to persist data as documents on PostgreSQL you often want a way to map these documents on structs.
//!
//! If you have such needs and you are using PostgreSQL instead of other databases, it's probably because you also want all other cool stuff present in PostgreSQL.
//!
//! **orma** is a special ORM for PostgreSQL written in Rust language developed just for you!
//!
//! **orma** takes advantage of PostgreSQL JSONB data representation and doesn't give up the relational advantages of a RDBMS like PostgreSQL.
//!
//! **orma** provides out of the box features for search and CRUD operation over your documents
//!
//! **orma** is fast and easy to learn, with a very simple API.
//!
//! ## Quick start
//!
//! **orma** is thought to be mapped over tables that have at least the following columns:
//!
//! ```disable
//! - id: uuid NOT NULL
//! - version: integer NOT NULL
//! - data: jsonb
//! ```
//!
//! - _id_ identifies a record.
//! - _version_ is used for record versioning and prevents a record to be modified if version has changed in another session. orma takes care of record versioning OOTB.
//! - _data_ is used to map the document and it's mapped over structs that implement _DbData_ trait.
//!
//! All structs that implement *DbData* trait need to be serializable too, as they'll be represented as jsonb data in your db records.
//!
//! While _DbData_ is mapped to _data_ column, _DbEntity_ is mapped over the three columns, and table, just described.
//!
//! You can use _DbEntity\<DbData\>_ to perform search and crud operations.
//!
//! If you need to relate two different entities, _DbJoin\<DbData\>_ is used to get entities related to the specified DbData.\
//! You need to use _JoinBuilder_ to create a DbJoin relation, and you can define both simple joins and table joins for M to N relations.
//!
//! # Example
//!
//! Using an imaginary "pgactix" database.
//!
//! ```ignore
//! use orma_derive::DbData;
//! use serde_derive::Serialize;
//! use orma::*;
//!
//! async fn create_connection() -> Connection {
//!     let connection_string = format!(
//!         "host={host} port={port} dbname={dbname} user={user} password={password}",
//!         host = &env::var("INTRARED_DB_HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
//!         port = env::var("INTRARED_DB_PORT").unwrap_or_else(|_| "5433".to_string()),
//!         dbname = env::var("INTRARED_DB_NAME").unwrap_or_else(|_| "pgactix".to_string()),
//!         user = env::var("INTRARED_DB_USERNAME").unwrap_or_else(|_| "pgactix".to_string()),
//!         password = env::var("INTRARED_DB_PASSWORD").unwrap_or_else(|_| "pgactix".to_string()),
//!     );
//!     let (client, conn) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls)
//!         .await
//!         .unwrap();
//!     tokio::spawn(conn);
//!     client.into()
//! }
//!
//! #[pk(field_1 = "field1", field_2 = "field2")]
//! #[table("schema.table_name")]
//! #[derive(Serialize, DbData)]
//! struct TestData {
//!     field_1: String,
//!     field_2: String,
//!     some_other_filed: Option<Vec<String>>,
//! }
//! ```

mod db_anti_corruption;
mod dbentity;
mod dbjoin;

pub use db_anti_corruption::tls;
pub use db_anti_corruption::*;
pub use dbentity::{select_part, DbData, DbEntity};
pub use dbjoin::{DbJoin, JoinBuilder};
pub use uuid::Uuid;

#[macro_export]
macro_rules! new_data {
    ($the_struct:ident, {$($body:tt)*}) => {
        $the_struct {
            orma_id: None,
            orma_version: None,
            $($body)*
        }
    }
}

#[cfg(feature = "derive")]
pub use orma_derive::*;
