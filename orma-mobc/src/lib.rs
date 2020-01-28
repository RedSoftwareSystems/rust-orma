use async_trait::async_trait;
use mobc::Manager;
use orma::tls::{MakeTlsConnect, TlsConnect};
use orma::{Config, Connection, DbError, Socket};

/// A `mobc::Manager` for `orma::Connection`s.
/// This crate is a strict derivation of [mobc-postgres](https://github.com/importcjj/mobc-postgres/)
///
/// ## Example
///
/// ```no_run
/// use mobc::Pool;
/// use orma_mobc::PgConnectionManager;
/// use orma::{Config, NoTls};
/// use std::time::Instant;
/// use std::str::FromStr;
/// #[tokio::main]
/// async fn main() {
///     let config = Config::from_str("postgres://user:passwd@localhost:5432").unwrap();
///     let manager = PgConnectionManager::new(config, NoTls);
///     let pool = Pool::builder().max_open(20).build(manager);
///     const MAX: usize = 5000;
///
///     let now = Instant::now();
///     let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(16);
///     for i in 0..MAX {
///         let pool = pool.clone();
///         let mut tx_c = tx.clone();
///         tokio::spawn(async move {
///             let connection = pool.get().await.unwrap();
///             let rows = connection.query("SELECT 1 + 2", &[]).await.unwrap();
///             let value: i32 = rows[0].get(0);
///             assert_eq!(value, 3);
///             tx_c.send(i).await.unwrap();
///         });
///     }
///     for _ in 0..MAX {
///         rx.recv().await.unwrap();
///     }
///
///     println!("cost: {:?}", now.elapsed());
/// }
/// ```
pub struct PgConnectionManager<Tls> {
    config: Config,
    tls: Tls,
}

impl<Tls> PgConnectionManager<Tls> {
    pub fn new(config: Config, tls: Tls) -> Self {
        Self { config, tls }
    }
}

#[async_trait]
impl<Tls> Manager for PgConnectionManager<Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    type Connection = Connection;
    type Error = DbError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tls = self.tls.clone();
        let (client, conn) = self.config.connect(tls).await?;
        mobc::spawn(conn);
        Ok(client.into())
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        conn.simple_query("").await?;
        Ok(conn)
    }
}
