use std::ops::Deref;

/// wrapper for tokio_postgres::Row
pub struct Row(tokio_postgres::Row);

impl Deref for Row {
    type Target = tokio_postgres::Row;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<tokio_postgres::Row> for Row {
    fn from(row: tokio_postgres::Row) -> Self {
        Self(row)
    }
}
