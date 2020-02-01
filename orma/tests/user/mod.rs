use crate::group::Group;
use orma::*;

use serde::{Deserialize, Serialize};

#[orma_obj(table = "intrared.users")]
#[derive(Serialize, Deserialize)]
pub struct User {
    pub user_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub user_name: String,
}

impl User {
    pub async fn find_by_user_name(
        db_conn: &Connection,
        user_name: &str,
    ) -> Result<Option<DbEntity<User>>, DbError> {
        DbEntity::<User>::find_by(db_conn, ("data->>'user_name'=$1", &[&user_name])).await
    }

    pub async fn find_all_sort(db_conn: &Connection) -> Result<Vec<DbEntity<User>>, DbError> {
        DbEntity::<User>::find_all(
            db_conn,
            None,
            Some(&["data->>'user_name' ASC", "data->>'email' DESC"]),
            0,
            1000,
        )
        .await
    }
}

pub async fn user_groups(
    user: &DbEntity<User>,
    db_conn: &Connection,
) -> Result<DbJoin<Group>, DbError> {
    JoinBuilder::new(&user.data)
        .with_join_table("intrared.r_user_group", "id_user", "id_group")
        .with_target(Group::table_name())
        .with_sorting(&["data->>'name'"])
        .build(db_conn, true)
        .await
}
