use crate::db_anti_corruption::Connection;
use crate::*;

use tokio_postgres::types::ToSql;
use uuid::Uuid;

/// This struct is used to create a join between DbEntyty elements
/// and JoinBuilder is used to create a DbJoin instance.
///
/// *sorting* parameter (such as vec!["data->>'first_name'", "data->>'last_name' DESC"]) take effect after a
/// call to *fetch* method.
///
/// *items* can be persisted to db after a call to *save_items* method.
///
/// See [JoinBuilder](struct.JoinBuilder.html) for more info
pub struct DbJoin {
    source_table: String,
    source_id: Uuid,
    source_fk: String,
    target_table: String,
    join_table: Option<String>,
    items_fk: Option<String>,
    /// The offset of the extracted query join
    pub offset: i64,
    /// The limit of the extracted query join. If limit is negative it means "no limit"
    pub limit: i64,
    /// Used to provide a sorting method on data fetching.
    /// ```rust
    /// vec!["data->>'first_name'", "data->>'last_name' DESC"];
    /// ```
    pub sorting: Vec<String>,
}

impl DbJoin {
    async fn load_items_simple_join<A>(
        &self,
        conn: &Connection,
        filter: Option<(&str, &[&(dyn ToSql + Sync)])>,
    ) -> Result<Vec<DbEntity<A>>, DbError>
    where
        A: DbData,
    {
        let qry = format!(
            "{A}, {source_table} b WHERE b.id = a.{b_fk}
            {filter}
            AND b.id = ${id_index}
            {order_by}{offset}{limit}",
            A = dbentity::select_part(&self.target_table, false, Some("a")),
            source_table = &self.source_table,
            b_fk = &self.source_fk,
            filter = match filter {
                Some((filter, _)) => format!("AND {}", filter.to_string()),
                _ => "".to_string(),
            },
            id_index = match filter {
                Some((_, filter)) => filter.len() + 1,
                _ => 1,
            },
            order_by = if !self.sorting.is_empty() {
                let order_by: Vec<&str> = self.sorting.iter().map(|x| x.as_ref()).collect();
                format!(
                    " ORDER BY {}",
                    dbentity::make_sort_statement(&order_by[..], Some("a"))
                )
            } else {
                "".to_string()
            },
            offset = format!(
                " OFFSET ${}",
                match filter {
                    Some(filter) => filter.1.len() + 2,
                    None => 2,
                }
            ),
            limit = if self.limit < 0 {
                "".to_string()
            } else {
                format!(
                    " LIMIT ${}",
                    match filter {
                        Some(filter) => filter.1.len() + 3,
                        None => 1,
                    }
                )
            },
        );
        let p_statement = conn.prepare(&qry).await?;
        let id = self.source_id;
        match filter {
            Some((_, filter_val)) => {
                let mut filter = filter_val.to_vec();
                let filter_id: Vec<&(dyn ToSql + Sync)> = vec![&id];
                filter.extend(filter_id.iter());
                filter.push(&self.offset as &(dyn ToSql + Sync));
                if self.limit >= 0 {
                    filter.push(&self.limit as &(dyn ToSql + Sync));
                }
                let result = conn.query(&p_statement, &filter).await?;
                Ok(DbEntity::from_rows(&result)?)
            }
            _ => {
                let filter: Vec<&(dyn ToSql + Sync)> = if self.limit < 0 {
                    vec![&id, &self.offset as &(dyn ToSql + Sync)]
                } else {
                    vec![
                        &id,
                        &self.offset as &(dyn ToSql + Sync),
                        &self.limit as &(dyn ToSql + Sync),
                    ]
                };
                let result = conn.query(&p_statement, &filter).await?;
                Ok(DbEntity::from_rows(&result)?)
            }
        }
    }

    async fn load_items_table_join<A>(
        &self,
        conn: &Connection,
        filter: Option<(&str, &[&(dyn ToSql + Sync)])>,
    ) -> Result<Vec<DbEntity<A>>, DbError>
    where
        A: DbData,
    {
        let qry = format!(
            "{A}, {join_table} ab, {source_table} b WHERE a.id = ab.{a_fk} AND b.id = ab.{b_fk}
                        {filter}
                        AND b.id = ${id_index}
                        {order_by}{offset}{limit}",
            A = dbentity::select_part(&self.target_table, false, Some("a")),
            join_table = self.join_table.as_ref().unwrap(),
            source_table = self.source_table,
            a_fk = self.items_fk.as_ref().unwrap(),
            b_fk = self.source_fk,
            filter = match filter {
                Some((filter, _)) => format!("AND {}", filter.to_string()),
                _ => "".to_string(),
            },
            id_index = match filter {
                Some((_, filter)) => filter.len() + 1,
                _ => 1,
            },
            order_by = if !self.sorting.is_empty() {
                let order_by: Vec<&str> = self.sorting.iter().map(|x| x.as_ref()).collect();
                format!(
                    " ORDER BY {}",
                    dbentity::make_sort_statement(&order_by[..], Some("a"))
                )
            } else {
                "".to_string()
            },
            offset = format!(
                " OFFSET ${}",
                match filter {
                    Some(filter) => filter.1.len() + 2,
                    None => 2,
                }
            ),
            limit = if self.limit < 0 {
                "".to_string()
            } else {
                format!(
                    " LIMIT ${}",
                    match filter {
                        Some(filter) => filter.1.len() + 3,
                        None => 1,
                    }
                )
            },
        );

        let p_statement = conn.prepare(&qry).await?;
        let id = self.source_id;
        match filter {
            Some((_, filter_val)) => {
                let mut filter = filter_val.to_vec();
                let filter_id: Vec<&(dyn ToSql + Sync)> = vec![&id];
                filter.extend(filter_id.iter());
                filter.push(&self.offset as &(dyn ToSql + Sync));
                if self.limit >= 0 {
                    filter.push(&self.limit as &(dyn ToSql + Sync));
                }
                let result = conn.query(&p_statement, &filter).await?;
                Ok(DbEntity::from_rows(&result)?)
            }
            _ => {
                let filter: Vec<&(dyn ToSql + Sync)> = if self.limit < 0 {
                    vec![&id, &self.offset as &(dyn ToSql + Sync)]
                } else {
                    vec![
                        &id,
                        &self.offset as &(dyn ToSql + Sync),
                        &self.limit as &(dyn ToSql + Sync),
                    ]
                };
                let result = conn.query(&p_statement, &filter).await?;
                Ok(DbEntity::from_rows(&result)?)
            }
        }
    }

    /// This method fetches items field for the given join using the current sorting field.
    pub async fn fetch<A>(&self, conn: &Connection) -> Result<Vec<DbEntity<A>>, DbError>
    where
        A: DbData,
    {
        let items = match (self.join_table.as_ref(), self.items_fk.as_ref()) {
            (Some(_join_table), Some(_items_fk)) => self.load_items_table_join(conn, None).await?,
            _ => self.load_items_simple_join(conn, None).await?,
        };
        Ok(items)
    }

    /// This method fetches items field for the given join using the current sorting field.
    /// Filter is a tuple with the query filter and its values.
    /// The target table is aliased with "a" name so a query filter could be `"a.data->>'name' LIKE $1"`
    pub async fn fetch_filtered<A>(
        &self,
        conn: &Connection,
        filter: (&str, &[&(dyn ToSql + Sync)]),
    ) -> Result<Vec<DbEntity<A>>, DbError>
    where
        A: DbData,
    {
        let items = match (self.join_table.as_ref(), self.items_fk.as_ref()) {
            (Some(_join_table), Some(_items_fk)) => {
                self.load_items_table_join(conn, Some(filter)).await?
            }
            _ => self.load_items_simple_join(conn, Some(filter)).await?,
        };
        Ok(items)
    }

    async fn remove_items_table_join_by_id(
        &self,
        conn: &Connection,
        items_id: Option<&[Uuid]>,
    ) -> Result<(), DbError> {
        if items_id.is_some() {
            let delete_sql = format!(
                "DELETE FROM {join_table} WHERE {items_fk} = ANY($1)",
                join_table = self
                    .join_table
                    .as_ref()
                    .ok_or_else(|| DbError::new("No join_table defined", None))?,
                items_fk = self
                    .items_fk
                    .as_ref()
                    .ok_or_else(|| DbError::new("No items_fk defined", None))?,
            );
            let delete_qry = conn.prepare(&delete_sql).await?;
            conn.execute(&delete_qry, &[&items_id]).await?;
        } else {
            let delete_sql = format!(
                "DELETE FROM {join_table} WHERE {source_fk} = $1",
                join_table = self
                    .join_table
                    .as_ref()
                    .ok_or_else(|| DbError::new("No join_table defined", None))?,
                source_fk = self.source_fk
            );
            let delete_qry = conn.prepare(&delete_sql).await?;
            let id = self.source_id;
            conn.execute(&delete_qry, &[&id]).await?;
        };

        Ok(())
    }

    async fn add_items_table_join_by_id(
        &self,
        conn: &Connection,
        items_id: &[&Uuid],
    ) -> Result<(), DbError> {
        let insert_val_sql = format!(
            "INSERT INTO {table_name} ({a_fk}, {b_fk}) VALUES {values} ON CONFLICT DO NOTHING",
            table_name = self
                .join_table
                .as_ref()
                .ok_or_else(|| DbError::new("No join_table defined", None))?
                .to_string(),
            b_fk = self
                .items_fk
                .as_ref()
                .ok_or_else(|| DbError::new("No items_fk defined", None))?,
            a_fk = self.source_fk,
            values = (0..items_id.len())
                .map(|i| format!("(${}, ${})", (i * 2) + 1, (i * 2) + 2))
                .collect::<Vec<String>>()
                .join(", ")
        );

        let insert_val_qry = conn.prepare(&insert_val_sql).await?;

        let ids_to_add: Vec<Vec<Uuid>> = items_id
            .to_vec()
            .into_iter()
            .map(|item_id| -> Vec<Uuid> {
                let source_id = self.source_id;
                let item_id = item_id.clone();
                vec![source_id, item_id]
            })
            .collect::<Vec<Vec<Uuid>>>();

        let ids_to_add = ids_to_add
            .iter()
            .flatten()
            .map(|item| item as &(dyn ToSql + Sync))
            .collect::<Vec<&(dyn ToSql + Sync)>>();

        conn.execute(&insert_val_qry, &ids_to_add).await?;
        Ok(())
    }

    async fn remove_items_simple_join_by_id(
        &self,
        conn: &Connection,
        items_id: Option<&[Uuid]>,
    ) -> Result<(), DbError> {
        let id = self.source_id;
        if items_id.is_some() {
            let delete_sql = format!(
                "UPDATE FROM {target_table} SET {a_fk} = NULL WHERE {a_fk} = $1 AND id = ANY($2)",
                target_table = self.target_table,
                a_fk = self.source_fk
            );
            let delete_qry = conn.prepare(&delete_sql).await?;

            conn.execute(&delete_qry, &[&id, &items_id]).await?;
        } else {
            let delete_sql = format!(
                "UPDATE FROM {target_table} SET {a_fk} = NULL WHERE {a_fk} = $1",
                target_table = self.target_table,
                a_fk = self.source_fk
            );
            let delete_qry = conn.prepare(&delete_sql).await?;
            conn.execute(&delete_qry, &[&id]).await?;
        };

        Ok(())
    }

    async fn add_items_simple_join_by_id(
        &self,
        conn: &Connection,
        items_id: &[&Uuid],
    ) -> Result<(), DbError> {
        let update_sql = format!(
            "UPDATE FROM {target_table} SET {a_fk} = $1 WHERE id =  ANY($2)",
            target_table = self.target_table,
            a_fk = self.source_fk
        );

        let update_qry = conn.prepare(&update_sql).await?;

        let id = self.source_id;

        conn.execute(&update_qry, &[&id, &items_id]).await?;

        Ok(())
    }

    /// This method adds items for the given join to the DB.
    pub async fn add_items<A>(
        &self,
        conn: &mut Connection,
        items: &[&DbEntity<A>],
    ) -> Result<(), DbError>
    where
        A: DbData,
    {
        let ids_to_add: Vec<Uuid> = items
            .iter()
            .map(|item| -> Result<Uuid, DbError> {
                item.id()
                    .ok_or_else(|| DbError::new("Item not persisted. No ID defined!", None))
            })
            .collect::<Result<Vec<Uuid>, DbError>>()?;

        self.add_items_by_id(conn, &ids_to_add.iter().collect::<Vec<&Uuid>>())
            .await
    }

    /// This method adds items for the given join to the DB using object ids.
    pub async fn add_items_by_id(
        &self,
        conn: &mut Connection,
        items: &[&Uuid],
    ) -> Result<(), DbError> {
        let result = async {
            match (self.join_table.as_ref(), self.items_fk.as_ref()) {
                (Some(_), Some(_)) => {
                    self.add_items_table_join_by_id(conn, &items).await?;
                }
                _ => {
                    self.add_items_simple_join_by_id(conn, &items).await?;
                }
            };
            Ok(())
        }
        .await;
        if result.is_err() {
            conn.rollback().await?;
        }
        result
    }

    /// This method removes items for the given join to the DB.
    pub async fn remove_items<A>(
        &self,
        conn: &mut Connection,
        items: Option<&[&DbEntity<A>]>,
    ) -> Result<(), DbError>
    where
        A: DbData,
    {
        if let Some(items) = items {
            let ids_to_remove: Vec<Uuid> = items
                .into_iter()
                .map(|item| -> Result<Uuid, DbError> {
                    item.id()
                        .ok_or_else(|| DbError::new("Item not persisted. No ID defined!", None))
                })
                .collect::<Result<Vec<Uuid>, DbError>>()?;
            self.remove_items_by_id(conn, Some(&ids_to_remove)).await
        } else {
            self.remove_items_by_id(conn, None).await
        }
    }

    /// This method removes items for the given join to the DB using object ids.
    pub async fn remove_items_by_id(
        &self,
        conn: &mut Connection,
        items_id: Option<&[Uuid]>,
    ) -> Result<(), DbError> {
        match (self.join_table.as_ref(), self.items_fk.as_ref()) {
            (Some(_), Some(_)) => {
                self.remove_items_table_join_by_id(conn, items_id).await?;
            }
            _ => {
                self.remove_items_simple_join_by_id(conn, items_id).await?;
            }
        };
        Ok(())
    }
}

pub struct JoinBuilder<'a, A>
where
    A: DbData,
{
    source: &'a A,
    source_fk: Option<&'a str>,
    target_table: Option<&'a str>,
    join_table: Option<&'a str>,
    items_fk: Option<&'a str>,
    sorting: &'a [&'a str],
}

impl<'a, A> JoinBuilder<'a, A>
where
    A: DbData,
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
    pub fn build(&self) -> Result<DbJoin, DbError> {
        Ok(DbJoin {
            source_table: A::table_name().to_string(),
            source_id: self
                .source
                .id()
                .ok_or_else(|| DbError::new("Source entity has no ID", None))?,
            source_fk: self.source_fk.expect("no source_fk defined").to_string(),
            target_table: self
                .target_table
                .ok_or_else(|| DbError::new("Source entity has no target_table", None))?
                .to_string(),
            join_table: self.join_table.map(String::from),
            items_fk: self.items_fk.map(String::from),
            sorting: self
                .sorting
                .iter()
                .map(|&item| item.to_string())
                .collect::<Vec<String>>(),
            offset: 0,
            limit: -1,
        })
    }
}
