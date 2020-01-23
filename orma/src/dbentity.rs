use crate::db_anti_corruption::Connection;
use crate::*;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::ops::Deref;
use uuid::Uuid;

/// Helper function to create a SQL SELECT statement for a DbEntity table.
/// This method reurns the (id, version, data) tuple
pub fn select_part(table_name: &str, alias: Option<&str>) -> String {
    let alias = if let Some(alias) = alias {
        format!("{}.", alias)
    } else {
        "".to_owned()
    };
    format!(
        "SELECT {alias}id, {alias}version, {alias}data FROM {table_name}",
        table_name = table_name,
        alias = alias
    )
}

/// Helper function to create the a SQL ORDER BY statement.
///
/// *order_by* clauses are joined by ", " and *alias* is prepended with "."
/// to each *order_by* element.
pub fn make_sort_statement(order_by: &[&str], alias: Option<&str>) -> String {
    match alias {
        Some(alias) => order_by
            .iter()
            .map(|&item| format!("{}.{}", alias, item))
            .collect::<Vec<String>>()
            .join(", "),
        _ => order_by.join(", "),
    }
}

/// This trait is maps data in a data table and
/// it's used along with DbEntity structure
pub trait DbData {
    /// The name of the db table where the implementing struct is mapped to.
    fn table_name() -> &'static str;

    /// Table name from instance
    fn table_name1(&self) -> &'static str {
        Self::table_name()
    }
    /// Convenience function that returns the select part for the associated db table.
    fn select_part() -> String {
        select_part(Self::table_name(), None)
    }

    /// Select part from instance
    fn select_part1(&self) -> String {
        Self::select_part()
    }

    /// In a data table each record is unique by at least a set of keys present in its data column.\n
    /// This pk is used to retrieve the id and version of the table record.
    fn pk_filter(&self) -> Vec<(&str, &(dyn ToSql + Sync))>;
}

/// This struct isused to create a mapping for a data table.
pub struct DbEntity<T>
where
    T: DbData + Serialize + DeserializeOwned,
{
    /// This is the effective prmary key of the record. Its also used to buildrelations with other tables
    pub id: Uuid,
    /// This field is used as a record check and identifies possible conflicts for parallel operations.
    /// verion should always autoinc on record update
    pub version: i32,
    /// The real information that a data table record is containing
    pub data: T,
}

impl<T> DbEntity<T>
where
    T: DbData + Serialize + DeserializeOwned,
{
    /// Simplemetod used to create a new record
    pub fn new(id: Uuid, version: i32, data: T) -> Self {
        Self { id, version, data }
    }

    pub async fn find_table_id_and_version_from_data(
        data: &T,
        conn: &Connection,
    ) -> Result<Option<(Uuid, i32)>, DbError> {
        let pk_filter = data.pk_filter();
        let pk_len = pk_filter.len();
        let (_, where_clauses, where_values): (u32, String, Vec<&(dyn ToSql + Sync)>) =
            pk_filter.into_iter().fold(
                (
                    0,
                    String::from(""),
                    Vec::<&(dyn ToSql + Sync)>::with_capacity(pk_len),
                ),
                |(c, where_str, mut where_values), kv| {
                    where_values.push(kv.1);
                    (
                        { c + 1 },
                        format!(
                            "{} {} data->>'{}' = ${}",
                            where_str,
                            if c == 0 { "" } else { " AND " },
                            kv.0,
                            c + 1
                        ),
                        where_values,
                    )
                },
            );

        let prepared_s = conn
            .prepare(&format!(
                "SELECT id, version FROM {table} WHERE {where_clause}",
                table = T::table_name(),
                where_clause = where_clauses
            ))
            .await?;
        let result = conn.query(&prepared_s, &where_values[..]).await?;
        if result.is_empty() {
            Ok(None)
        } else {
            let row = &result.get(0).unwrap();
            let uuid: Uuid = row.get(0);
            let version: i32 = row.get(1);
            Ok(Some((uuid, version)))
        }
    }

    /// Given a data this method uses DbData#find_table_id_and_version to find a possible candidate for record or creates
    /// a new one that will need to be persisted with the insert method.
    pub async fn from_data<'a>(data: T, conn: &Connection) -> Result<Self, DbError> {
        let uuid_and_version = Self::find_table_id_and_version_from_data(&data, conn).await?;
        match uuid_and_version {
            Some((uuid, version)) => Ok(Self {
                id: uuid,
                version,
                data,
            }),
            None => Ok(Self {
                id: Uuid::new_v4(),
                version: 0,
                data,
            }),
        }
    }

    /// Given a database row (id, version, data) returns a DbEntity.
    pub fn from_row(row: &Row) -> Result<Self, DbError> {
        let uuid: Uuid = row.get(0);
        let version: i32 = row.get(1);
        let data: T = serde_json::from_value(row.get::<_, serde_json::Value>(2))
            .map_err(DbError::from)
            .unwrap();
        Ok(DbEntity::new(uuid, version, data))
    }

    /// Given a database rows of (id, version, data) tuples returns a Vec of DbEntity.
    pub fn from_rows(rows: &[Row]) -> Result<Vec<Self>, DbError> {
        if rows.is_empty() {
            Ok(vec![])
        } else {
            Ok(rows
                .iter()
                .map(|row| DbEntity::from_row(&row).unwrap())
                .collect())
        }
    }

    fn out_of_sync_err(&self) -> DbError {
        DbError::new(&format!("{}:{} out of sync", self.id, self.version), None)
    }

    /// Inserts a new record into the associated table
    pub async fn insert<'a>(&mut self, conn: &Connection) -> Result<(), DbError> {
        let prepared_s = conn
            .prepare(&format!(
                "INSERT INTO  {table_name} (id, version, data) VALUES ($1, $2+1, $3)",
                table_name = T::table_name()
            ))
            .await?;
        conn.execute(
            &prepared_s,
            &[
                &self.id,
                &self.version,
                &serde_json::to_value(&self.data).unwrap(),
            ],
        )
        .await?;
        self.version += 1;
        Ok(())
    }

    /// Persists the record
    pub async fn update(&mut self, conn: &Connection) -> Result<(), DbError> {
        let prepared_s = conn
            .prepare(&format!(
                "UPDATE {table_name} SET
            version=$2+1,
            data=$3
            WHERE
            id = $1 AND
            version = $2",
                table_name = T::table_name()
            ))
            .await?;
        let updated = conn
            .execute(
                &prepared_s,
                &[
                    &self.id,
                    &self.version,
                    &serde_json::to_value(&self.data).unwrap(),
                ],
            )
            .await?
            == 1;
        if updated {
            self.version += 1;
            Ok(())
        } else {
            Err(self.out_of_sync_err())
        }
    }

    /// Performs a record deletion
    pub async fn delete(&mut self, conn: &Connection) -> Result<(), DbError> {
        let prepared_s = conn
            .prepare(&format!(
                "DELETE FROM {table_name}
            WHERE
            id = $1 AND
            version = $2",
                table_name = T::table_name()
            ))
            .await?;
        let deleted = conn
            .execute(&prepared_s, &[&self.id, &self.version])
            .await?
            == 1;
        if deleted {
            self.version = 0;
            Ok(())
        } else {
            Err(self.out_of_sync_err())
        }
    }

    /// Searches for a record where filter over data column (JSONB) matches provided parameters.
    /// ## Example
    /// ```ignore
    /// DbEntity::<User>::find_by(db_conn, ("data->>'user_name'=$1", &["some_name"]));
    /// ```
    pub async fn find_by(
        conn: &Connection,
        filter: (&str, &[&(dyn ToSql + Sync)]),
    ) -> Result<Option<Self>, DbError> {
        let prepared_s = conn
            .prepare(&format!(
                "{select_part} WHERE {filter}",
                select_part = T::select_part(),
                filter = filter.0,
            ))
            .await?;

        let result = conn.query(&prepared_s, filter.1).await?;
        if result.is_empty() {
            Ok(None)
        } else {
            let row = &result.get(0).unwrap();
            DbEntity::from_row(&row).map(Some)
        }
    }

    /// Searching all matching records defined by filtre clause (first element of the filter tuple)\
    /// A sorting clause can be given.\
    /// Limit and offset define the perimeter of the query result.
    /// ## Example
    /// ```ignore
    /// DbEntity::<User>::find_all(
    ///    db_conn,
    ///    (
    ///        "data->>'user_name'=$1",
    ///        &["some_name"],
    ///        Some(&["data->>'user_name' DESC"]),
    ///        0,
    ///        100,
    ///    ),
    /// );
    /// ```
    pub async fn find_all(
        conn: &Connection,
        filter: Option<(&str, &[&(dyn ToSql + Sync)])>,
        sorting: Option<&[&str]>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, DbError> {
        let prepared_s = conn.prepare(&format!(
            "{select_part}{where}{sorting}{limit}{offset}",
            select_part = T::select_part(),
            where = match filter {
                Some(where_clause) => format!(" WHERE {}",where_clause.0),
                None => String::from(""),
            },
            sorting = match sorting {
                Some(sorting_statement) => format!(" ORDER BY {}", make_sort_statement(sorting_statement, None) ),
                None => String::from("")
            },
            limit = format!(" LIMIT ${}", match filter {
                    Some(filter) => filter.1.len() + 1,
                    None => 1
            }),
            offset = format!(" OFFSET ${}", match filter {
                    Some(filter) => filter.1.len() + 2,
                    None => 2
            }),
        )).await?;

        let params: Vec<&(dyn ToSql + Sync)> = match filter {
            Some(filter) => [filter.1, &[&limit, &offset]].concat(),
            None => vec![&limit, &offset],
        };
        let result = conn.query(&prepared_s, &params[..]).await?;
        DbEntity::from_rows(&result)
    }
}

impl<T> Deref for DbEntity<T>
where
    T: DbData + Serialize + DeserializeOwned,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct User {
        pub user_name: String,
        pub first_name: String,
        pub last_name: String,
    }

    impl DbData for User {
        fn table_name() -> &'static str {
            "intrared.users"
        }

        fn pk_filter(&self) -> Vec<(&str, &(dyn ToSql + Sync))> {
            vec![("user_name", &self.user_name as &(dyn ToSql + Sync))]
        }
    }

    impl User {
        fn new(user_name: &str, first_name: &str, last_name: &str) -> Self {
            User {
                user_name: user_name.to_string(),
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
            }
        }
    }

    #[test]
    fn test_schema_instance_retrival() {
        let user1 = User::new("user_name", "Jhon", "Doe");

        assert!(user1.table_name1() == User::table_name());
    }

    #[test]
    fn test_dbentity_deref() {
        let full_name =
            |user: &User| -> String { format!("{} {}", user.last_name, user.first_name) };

        let entity_status =
            |entity: &DbEntity<User>| -> String { format!("{}:{}", entity.id, entity.version) };

        let uuid = Uuid::new_v4();
        let data = User::new("user_name", "Jhon", "Doe");

        let expected_status = format!("{}:0", uuid);
        let expected_full_name = full_name(&data);

        let user_dbe = DbEntity::new(uuid, 0, data);

        assert_eq!(entity_status(&user_dbe), expected_status);
        assert_eq!(full_name(&user_dbe), expected_full_name);
    }

    #[test]
    fn test_select_extra_columns() {
        struct Test {
            attr: String,
        };

        impl DbData for Test {
            fn table_name() -> &'static str {
                "intrared.test"
            }

            fn select_part() -> String {
                format!(
                    "{}, {}",
                    select_part(Self::table_name(), None),
                    "another_col"
                )
            }

            fn pk_filter(&self) -> Vec<(&str, &(dyn ToSql + Sync))> {
                vec![("attr", &self.attr as &(dyn ToSql + Sync))]
            }
        };
        let t = Test {
            attr: "attr".to_string(),
        };

        assert_eq!(Test::select_part(), t.select_part1());
    }
}
