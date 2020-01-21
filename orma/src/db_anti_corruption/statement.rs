use std::ops::Deref;

mod private {
    use crate::{Connection, DbError, Statement};
    pub trait Sealed {}

    pub enum ToStatementType<'a> {
        Statement(&'a Statement),
        Query(&'a str),
    }

    impl<'a> ToStatementType<'a> {
        pub async fn into_statement(self, conn: &Connection) -> Result<Statement, DbError> {
            match self {
                ToStatementType::Statement(s) => Ok(s.clone()),
                ToStatementType::Query(s) => conn.prepare(s).await,
            }
        }
    }
}

use private::{Sealed, ToStatementType};

/// Wrapper around tokio_postgres::Statement
#[derive(Clone)]
pub struct Statement(tokio_postgres::Statement);

impl Deref for Statement {
    type Target = tokio_postgres::Statement;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<tokio_postgres::Statement> for Statement {
    fn from(statement: tokio_postgres::Statement) -> Self {
        Self(statement)
    }
}

/// Like tokio_postgres::ToStatement trait
pub trait ToStatement: Sealed {
    #[doc(hidden)]
    fn __convert(&self) -> ToStatementType<'_>;
}

impl ToStatement for Statement {
    fn __convert(&self) -> ToStatementType<'_> {
        ToStatementType::Statement(self)
    }
}

impl Sealed for Statement {}

impl ToStatement for str {
    fn __convert(&self) -> ToStatementType<'_> {
        ToStatementType::Query(self)
    }
}

impl Sealed for str {}
