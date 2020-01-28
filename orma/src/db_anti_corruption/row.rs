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

/// wrapper for tokio_postgres::SimpleQueryRow
pub struct SimpleQueryRow(tokio_postgres::SimpleQueryRow);

impl Deref for SimpleQueryRow {
    type Target = tokio_postgres::SimpleQueryRow;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<tokio_postgres::SimpleQueryRow> for SimpleQueryRow {
    fn from(row: tokio_postgres::SimpleQueryRow) -> Self {
        Self(row)
    }
}
// Message returned by the `simple_query` stream.
pub enum SimpleQueryMessage {
    /// A row of data.
    Row(SimpleQueryRow),
    /// A statement in the query has completed.
    ///
    /// The number of rows modified or selected is returned.
    CommandComplete(u64),
}

impl From<tokio_postgres::SimpleQueryMessage> for SimpleQueryMessage {
    fn from(row: tokio_postgres::SimpleQueryMessage) -> Self {
        match row {
            tokio_postgres::SimpleQueryMessage::Row(row) => SimpleQueryMessage::Row(row.into()),
            tokio_postgres::SimpleQueryMessage::CommandComplete(count) => {
                SimpleQueryMessage::CommandComplete(count)
            }
            _ => SimpleQueryMessage::CommandComplete(0),
        }
    }
}
