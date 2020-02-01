use crate::db_anti_corruption::Connection;
use crate::*;
use futures::{future::join_all, try_join};

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

fn id_from_str(id: &str) -> Result<Uuid, DbError> {
    Uuid::parse_str(&id)
        .map_err(|e| DbError::new(&format!("Unable to parse, {} id", id), Some(Box::new(e))))
}

/// This struct is used to create a join between DbEntyty elements
/// and JoinBuilder is used to create a DbJoin instance.
///
/// *sorting* parameter (such as vec!["data->>'first_name'", "data->>'last_name' DESC"]) take effect after a
/// call to *fetch* method.
///
/// *items* can be persisted to db after a call to *save_items* method.
///
/// See [JoinBuilder](struct.JoinBuilder.html) for more info
pub struct DbJoin<A>
where
    A: DbData + Serialize + DeserializeOwned,
{
    source_table: String,
    source_id: String,
    source_fk: String,
    target_table: String,
    join_table: Option<String>,
    items_fk: Option<String>,
    /// Used to provide a sorting method on data fetching.
    /// ```rust
    /// vec!["data->>'first_name'", "data->>'last_name' DESC"];
    /// ```
    pub sorting: Vec<String>,
    /// Items for this relation over the DbEntity source that has been provided.
    pub items: Vec<DbEntity<A>>,
}

impl<A> DbJoin<A>
where
    A: DbData + Serialize + DeserializeOwned,
{
    async fn load_items_simple_join(&self, conn: &Connection) -> Result<Vec<DbEntity<A>>, DbError> {
        let qry = format!(
            "{B}, {source_table} a WHERE a.id = b.{a_fk} AND a.id = $1{order_by}",
            B = dbentity::select_part(&self.target_table, false, Some("b")),
            source_table = &self.source_table,
            a_fk = &self.source_fk,
            order_by = if !self.sorting.is_empty() {
                let order_by: Vec<&str> = self.sorting.iter().map(|x| x.as_ref()).collect();
                format!(
                    " ORDER BY {}",
                    dbentity::make_sort_statement(&order_by[..], None)
                )
            } else {
                "".to_string()
            }
        );
        let p_statement = conn.prepare(&qry).await?;
        let id = id_from_str(&self.source_id)?;
        let result = conn.query(&p_statement, &[&id]).await?;
        Ok(DbEntity::from_rows(&result).unwrap())
    }

    async fn load_items_table_join(&self, conn: &Connection) -> Result<Vec<DbEntity<A>>, DbError> {
        let qry = format!(
                        "{B}, {join_table} ab, {source_table} a WHERE b.id = ab.{b_fk} AND a.id = ab.{a_fk} AND a.id = $1{order_by}",
                        B = dbentity::select_part(&self.target_table, false, Some("b")),
                        join_table = self.join_table.as_ref().unwrap(),
                        source_table = self.source_table,
                        b_fk = self.items_fk.as_ref().unwrap(),
                        a_fk = self.source_fk,
                        order_by = if !self.sorting.is_empty() {
                            let order_by: Vec<&str> = self.sorting.iter().map(|x| x.as_ref()).collect();
                            format!(" ORDER BY {}",
                            dbentity::make_sort_statement(&order_by[..], Some("b")))
                        } else {
                            "".to_string()
                        }
                    );
        let p_statement = conn.prepare(&qry).await?;
        let id = id_from_str(&self.source_id)?;
        let result = conn.query(&p_statement, &[&id]).await?;
        Ok(DbEntity::from_rows(&result).unwrap())
    }

    /// This method fetches items field for the given join using the current sorting field.
    pub async fn fetch(&mut self, conn: &Connection) -> Result<(), DbError> {
        self.items = match (self.join_table.as_ref(), self.items_fk.as_ref()) {
            (Some(_join_table), Some(_items_fk)) => self.load_items_table_join(conn).await?,
            _ => self.load_items_simple_join(conn).await?,
        };
        Ok(())
    }

    async fn save_items_table_join(
        &self,
        join_table: &str,
        items_fk: &str,
        conn: &Connection,
    ) -> Result<(), DbError> {
        let delete_sql = format!(
            "DELETE FROM {join_table} WHERE {a_fk} = $1",
            join_table = &join_table,
            a_fk = self.source_fk
        );

        let delete_qry = conn.prepare(&delete_sql);

        let insert_val_sql = format!(
            "INSERT INTO {table_name} ({a_fk}, {b_fk}) VALUES ($1, $2)",
            table_name = &join_table,
            b_fk = &items_fk,
            a_fk = self.source_fk
        );
        let insert_val_qry = conn.prepare(&insert_val_sql);

        let (delete_qry, insert_val_qry) = try_join!(delete_qry, insert_val_qry)?;

        let id = id_from_str(&self.source_id)?;
        conn.execute(&delete_qry, &[&id]).await?;

        let params_owned: Vec<Vec<Uuid>> = self
            .items
            .iter()
            .map(|db_entity| {
                vec![
                    Uuid::parse_str(&self.source_id).unwrap(),
                    Uuid::parse_str(&format!("{}", db_entity.id)).unwrap(),
                ]
            })
            .collect();

        let params: Vec<Vec<_>> = params_owned
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|uuid_ref| uuid_ref as &(dyn ToSql + Sync))
                    .collect()
            })
            .collect();

        join_all(
            params
                .iter()
                .map(|q_params| conn.execute(&insert_val_qry, &q_params[..])),
        )
        .await;
        Ok(())
    }

    async fn save_items_simple_join(&self, conn: &Connection) -> Result<(), DbError> {
        let update_sql = format!(
            "UPDATE FROM {target_table} SET {a_fk} = NULL WHERE {a_fk} = $1",
            target_table = self.target_table,
            a_fk = self.source_fk
        );

        let update_qry = conn.prepare(&update_sql);

        let insert_val_sql = format!(
            "UPDATE FROM {target_table} SET {a_fk} = $1 WHERE id = $2",
            target_table = self.target_table,
            a_fk = self.source_fk
        );
        let insert_val_qry = conn.prepare(&insert_val_sql);
        let (update_qry, insert_val_qry) = try_join!(update_qry, insert_val_qry)?;

        let id = id_from_str(&self.source_id)?;
        conn.execute(&update_qry, &[&id]).await?;

        let params_owned: Vec<Vec<Uuid>> = self
            .items
            .iter()
            .map(|db_entity| {
                vec![
                    Uuid::parse_str(&self.source_id).unwrap(),
                    Uuid::parse_str(&format!("{}", db_entity.id)).unwrap(),
                ]
            })
            .collect();

        let params: Vec<Vec<_>> = params_owned
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|uuid_ref| uuid_ref as &(dyn ToSql + Sync))
                    .collect()
            })
            .collect();

        join_all(
            params
                .iter()
                .map(|q_params| conn.execute(&insert_val_qry, &q_params[..])),
        )
        .await;
        Ok(())
    }

    /// This method saves the items for the given join to the DB.
    pub async fn save_items(&self, conn: &mut Connection) -> Result<(), DbError> {
        conn.transaction().await?;
        let result = async {
            match (self.join_table.as_ref(), self.items_fk.as_ref()) {
                (Some(join_table), Some(items_fk)) => {
                    self.save_items_table_join(&join_table, &items_fk, conn)
                        .await?;
                }
                _ => {
                    self.save_items_simple_join(conn).await?;
                }
            };
            conn.commit().await?;
            Ok(())
        }
        .await;
        if result.is_err() {
            conn.rollback().await?;
        }
        result
    }
}

/// This is the builder for a [DbJoin](struct.DbJoin.html)
/// If you need to relate two different entities, _DbJoin\<DbData\>_ is used to get entities related to the specified DbData.\
/// You need to use _JoinBuilder_ to create a DbJoin relation, and you can define both simple joins and table joins for M to N relations.
///
/// ## Simple join example:
/// ```disable
/// async fn office_users(
///     user: &DbEntity<Office>,
///     db_conn: &Connection,
/// ) -> Result<DbJoin<User>, DbError> {
///     JoinBuilder::new(user.data)
///         .with_source_fk("id_office")
///         .with_target(User::table_name())
///         .with_sorting(&["data->>'name'"])
///         .build(db_conn)
///         .await
/// }
/// ```
///
/// ## M to N join example:
/// ```disable
/// async fn user_groups(
///     user: &DbEntity<User>,
///     db_conn: &Connection,
/// ) -> Result<DbJoin<Group>, DbError> {
///     JoinBuilder::new(user.data)
///         .with_join_table("intrared.r_user_group", "id_user", "id_group")
///         .with_target(Group::table_name())
///         .with_sorting(&["data->>'name'"])
///         .build(db_conn)
///         .await
/// }
/// ```

pub struct JoinBuilder<'a, A, B>
where
    A: DbData + Serialize + DeserializeOwned,
    B: DbData + Serialize + DeserializeOwned,
{
    source: &'a A,
    source_fk: Option<&'a str>,
    target_table: Option<&'a str>,
    join_table: Option<&'a str>,
    items_fk: Option<&'a str>,
    _items: Vec<DbEntity<B>>,
    sorting: &'a [&'a str],
}

impl<'a, A, B> JoinBuilder<'a, A, B>
where
    A: DbData + Serialize + DeserializeOwned,
    B: DbData + Serialize + DeserializeOwned,
{
    /// Defines the source DbEntity of the DbJoin
    pub fn new(source: &'a A) -> Self {
        Self {
            source,
            source_fk: None,
            target_table: None,
            join_table: None,
            items_fk: None,
            sorting: &[],
            _items: vec![],
        }
    }

    /// Provides the name of the table where DbJoin items are mapped to
    pub fn with_target(mut self, target_table: &'a str) -> Self {
        self.target_table = Some(target_table);
        self
    }
    /// The name of the source foreign key
    /// in the items table when you want to represents a simple join ( 1 to n )
    pub fn with_source_fk(mut self, source_fk: &'a str) -> Self {
        self.source_fk = Some(source_fk);
        self
    }

    /// If you want to map a m2n join, you provide the name of the db table with
    /// name of the source foreign key
    /// and the foreign key of the items table
    pub fn with_join_table(
        mut self,
        join_table: &'a str,
        source_fk: &'a str,
        items_fk: &'a str,
    ) -> Self {
        self.join_table = Some(join_table);
        self.source_fk = Some(source_fk);
        self.items_fk = Some(items_fk);
        self
    }

    /// DbJoin sorting attribute used to provide a sorting method on data fetching.
    /// ```rust
    /// vec!["data->>'first_name'", "data->>'last_name' DESC"];
    /// ```
    pub fn with_sorting(mut self, sorting: &'a [&'a str]) -> Self {
        self.sorting = sorting;
        self
    }

    /// Creates the DbJoin object and fetches the data
    pub async fn build(&self, conn: &'a Connection, fetch: bool) -> Result<DbJoin<B>, DbError> {
        let mut join = DbJoin {
            source_table: A::table_name().to_string(),
            source_id: format!("{}", self.source.id().expect("no id for source")),
            source_fk: self.source_fk.expect("no source_fk defined").to_string(),
            target_table: self
                .target_table
                .expect("no target_table defined")
                .to_string(),
            join_table: self.join_table.map(String::from),
            items_fk: self.items_fk.map(String::from),
            sorting: self
                .sorting
                .iter()
                .map(|&item| item.to_string())
                .collect::<Vec<String>>(),
            items: Vec::<DbEntity<B>>::new(),
        };
        if fetch {
            join.fetch(conn).await?;
        }
        Ok(join)
    }
}
