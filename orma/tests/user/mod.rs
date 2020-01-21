use crate::group::Group;
use orma::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub user_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub user_name: String,
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
    JoinBuilder::new(user)
        .with_join_table("intrared.r_user_group", "id_user", "id_group")
        .with_target(Group::table_name())
        .with_sorting(&["data->>'name'"])
        .build(db_conn)
        .await
}
