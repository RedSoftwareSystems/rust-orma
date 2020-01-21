use orma::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub description: Option<String>,
}

impl DbData for Group {
    fn table_name() -> &'static str {
        "intrared.groups"
    }

    fn pk_filter(&self) -> Vec<(&str, &(dyn ToSql + Sync))> {
        vec![("name", &self.name as &(dyn ToSql + Sync))]
    }
}

impl Group {
    pub async fn find_by_name(
        db_conn: &Connection,
        name: &str,
    ) -> Result<Option<DbEntity<Group>>, DbError> {
        DbEntity::<Group>::find_by(db_conn, ("data->>'name'=$1", &[&name])).await
    }
}
