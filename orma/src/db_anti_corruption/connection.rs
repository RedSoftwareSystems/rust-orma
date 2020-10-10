use super::{DbError, Row, SimpleQueryMessage, Statement, ToSql, ToStatement};
use tokio_postgres::Client;

/// Wrapper over tokio_postgres::Client
/// ## Example
/// ```edition2018
///  use std::env;
///  use orma::{Connection};
///
/// async fn create_connection() -> Connection {
///     let connection_string = format!(
///         "host={host} port={port} dbname={dbname} user={user} password={password}",
///         host = &env::var("INTRARED_DB_HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
///         port = env::var("INTRARED_DB_PORT").unwrap_or_else(|_| "5433".to_string()),
///         dbname = env::var("INTRARED_DB_NAME").unwrap_or_else(|_| "pgactix".to_string()),
///         user = env::var("INTRARED_DB_USERNAME").unwrap_or_else(|_| "pgactix".to_string()),
///         password = env::var("INTRARED_DB_PASSWORD").unwrap_or_else(|_| "pgactix".to_string()),
///     );
///     let (client, conn) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls)
///         .await
///         .unwrap();
///     tokio::spawn(conn);
///     Connection::from(client)
/// }
/// ```
pub struct Connection {
    client: Client,
    transaction_n: u32,
}

impl From<Client> for Connection {
    fn from(client: Client) -> Self {
        Self {
            client,
            transaction_n: 0,
        }
    }
}

impl Connection {
    /// Executes a sequence of SQL statements using the simple query protocol.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the
    /// sequence will stop at that point. This is intended for use when, for example, initializing
    /// a database schema.
    pub async fn batch_execute(&self, query: &str) -> Result<(), DbError> {
        self.client
            .batch_execute(query)
            .await
            .map_err(DbError::from)
    }

    /// Executes a statement, returning the number of rows modified.
    ///
    /// A statement may contain parameters, specified by $n, where n is the index of the parameter
    /// of the list provided, 1-indexed.
    ///
    /// The statement argument can either be a Statement, or a raw query string.
    /// If the same statement will be repeatedly executed (perhaps with different query parameters),
    /// consider preparing the statement up front with the prepare method.
    ///
    /// If the statement does not modify any rows (e.g. SELECT), 0 is returned.
    pub async fn execute<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, DbError>
    where
        T: ?Sized + ToStatement,
    {
        let statement = &statement.__convert().into_statement(self).await?;
        self.client
            .execute(&**statement, params)
            .await
            .map_err(DbError::from)
    }

    /// Creates a new prepared statement.
    ///
    /// Prepared statements can be executed repeatedly, and may contain query parameters
    /// (indicated by $1, $2, etc), which are set when executed.
    ///
    /// Prepared statements can only be used with the connection that created them.
    pub async fn prepare(&self, query: &str) -> Result<Statement, DbError> {
        self.client
            .prepare(query)
            .await
            .map(Statement::from)
            .map_err(DbError::from)
    }

    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, DbError> {
        self.client
            .simple_query(query)
            .await
            .map(|rows| rows.into_iter().map(SimpleQueryMessage::from).collect())
            .map_err(DbError::from)
    }

    /// Executes a statement, returning a vector of the resulting rows.
    ///
    /// A statement may contain parameters, specified by $n, where n is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The statement argument can either be a Statement, or a raw query string.
    /// If the same statement will be repeatedly executed (perhaps with different
    /// query parameters), consider preparing the statement up front with the prepare method.
    pub async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, DbError>
    where
        T: ?Sized + ToStatement,
    {
        let statement = &statement.__convert().into_statement(self).await?;
        self.client
            .query(&**statement, params)
            .await
            .map(|rows| rows.into_iter().map(Row::from).collect())
            .map_err(DbError::from)
    }

    /// Executes a statement, returning a single row.
    ///
    /// A statement may contain parameters, specified by $n, where n is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The statement argument can either be a Statement, or a raw query string.
    /// If the same statement will be repeatedly executed (perhaps with different
    /// query parameters), consider preparing the statement up front with the prepare method.
    pub async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, DbError>
    where
        T: ?Sized + ToStatement,
    {
        let statement = &statement.__convert().into_statement(self).await?;
        self.client
            .query_one(&**statement, params)
            .await
            .map(Row::from)
            .map_err(DbError::from)
    }

    /// Executes a statement, returning zero or one row.
    ///
    /// A statement may contain parameters, specified by $n, where n is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The statement argument can either be a Statement, or a raw query string.
    /// If the same statement will be repeatedly executed (perhaps with different
    /// query parameters), consider preparing the statement up front with the prepare method.
    pub async fn query_opt<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, DbError>
    where
        T: ?Sized + ToStatement,
    {
        let statement = &statement.__convert().into_statement(self).await?;
        self.client
            .query_opt(&**statement, params)
            .await
            .map(|option_row| match option_row {
                Some(row) => Some(Row::from(row)),
                _ => None,
            })
            .map_err(DbError::from)
    }

    /// Begins a transaction or creates a savepoint if a transaction already started
    /// ## Example
    /// from [dbjoin.rs](../src/orma/dbjoin.rs.html)
    /// ```disabled
    /// pub async fn save_items(&self, conn: &mut Connection) -> Result<(), DbError> {
    ///     conn.transaction().await?;
    ///     let result = async {
    ///         match (self.join_table.as_ref(), self.items_fk.as_ref()) {
    ///             (Some(join_table), Some(items_fk)) => {
    ///                 self.save_items_table_join(&join_table, &items_fk, conn)
    ///                     .await?;
    ///             }
    ///             _ => {
    ///                 self.save_items_simple_join(conn).await?;
    ///             }
    ///         };
    ///         conn.commit().await?;
    ///         Ok(())
    ///     }
    ///     .await;
    ///     if result.is_err() {
    ///         conn.rollback().await?;
    ///     }
    ///     result
    /// }
    /// ```
    pub async fn transaction(&mut self) -> Result<(), DbError> {
        let qry = if self.transaction_n == 0 {
            // String::from("BEGIN")
            "BEGIN".into()
        } else {
            format!("SAVEPOINT pt{}", self.transaction_n)
        };
        self.batch_execute(&qry).await?;
        self.transaction_n += 1;
        Ok(())
    }

    /// Commits a transaction or releases a savepoint
    pub async fn commit(&mut self) -> Result<(), DbError> {
        if self.transaction_n == 0 {
            Err(DbError::new("Not in a transaction", None))
        } else {
            let qry = if self.transaction_n == 1 {
                String::from("COMMIT")
            } else {
                format!("RELEASE pt{}", self.transaction_n - 1)
            };
            self.batch_execute(&qry).await?;
            self.transaction_n -= 1;
            Ok(())
        }
    }

    /// Rolls back a transaction or a savepoint
    pub async fn rollback(&mut self) -> Result<(), DbError> {
        if self.transaction_n == 0 {
            Err(DbError::new("Not in a transaction", None))
        } else {
            let qry = if self.transaction_n == 1 {
                String::from("ROLLBACK")
            } else {
                format!("ROLLBACK TO SAVEPOINT pt{}", self.transaction_n - 1)
            };
            self.batch_execute(&qry).await?;
            self.transaction_n -= 1;
            Ok(())
        }
    }
}
