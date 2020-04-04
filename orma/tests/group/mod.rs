use orma::*;
use serde::{Deserialize, Serialize};

#[orma_obj(table = "intrared.groups")]
#[derive(Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub description: Option<String>,
}

impl Group {
    pub async fn find_by_name(
        db_conn: &Connection,
        name: &str,
    ) -> Result<Option<DbEntity<Group>>, DbError> {
        DbEntity::<Group>::find_by(db_conn, ("data->>'name'=$1", &[&name])).await
    }
}
