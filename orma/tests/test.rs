pub mod group;
pub mod user;

use group::*;
use orma::*;
use std::env;
use user::*;

#[tokio::test]
async fn test() {}

pub async fn create_connection() -> Connection {
    let connection_string = format!(
        "host={host} port={port} dbname={dbname} user={user} password={password}",
        host = env::var("ORMA_DB_HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
        port = env::var("ORMA_DB_PORT").unwrap_or_else(|_| "5433".to_string()),
        dbname = env::var("ORMA_DB_NAME").unwrap_or_else(|_| "pgactix".to_string()),
        user = env::var("ORMA_DB_USERNAME").unwrap_or_else(|_| "pgactix".to_string()),
        password = env::var("ORMA_DB_PASSWORD").unwrap_or_else(|_| "pgactix".to_string()),
    );
    let (client, conn) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(conn);
    Connection::from(client)
}

pub async fn clear_tables(conn: &mut Connection) -> Result<(), DbError> {
    conn.transaction().await?;
    conn.batch_execute("DELETE from intrared.users").await?;
    conn.batch_execute("DELETE from intrared.groups").await?;
    conn.batch_execute("DELETE from intrared.r_user_group")
        .await?;
    conn.commit().await
}

fn create_user(user_name: &str, email: &str) -> User {
    User {
        user_name: user_name.to_owned(),
        email: email.to_owned(),
        user_id: Some(user_name.to_owned()),
        first_name: String::from("FirstName"),
        last_name: String::from("LastName"),
    }
}

fn create_group(name: &str, description: &str) -> Group {
    Group {
        name: name.to_owned(),
        description: Some(description.to_owned()),
    }
}

#[tokio::test]
async fn test_user_crud() {
    let mut conn = create_connection().await;

    clear_tables(&mut conn).await.unwrap();

    let user_name = "test_user_crud";
    let email = "test_user_crud@test.com";

    let user1 = create_user(user_name, email);

    let user_entity = &mut DbEntity::from_data(user1, &conn).await.unwrap();

    user_entity.insert(&conn).await.unwrap();

    let find_result = User::find_by_user_name(&conn, &user_name).await;
    assert!(
        find_result.is_ok(),
        format!("Failed to find user {}", &user_name)
    );
    assert!(
        find_result.unwrap().is_some(),
        format!("User {} not found", &user_name)
    );

    assert!(user_entity.delete(&conn).await.is_ok(), "Delete Failed");

    let find_after_delete_result = User::find_by_user_name(&conn, &user_name).await;

    assert!(
        find_after_delete_result.unwrap().is_none(),
        format!("User {} not deleted", &user_name)
    );
}

#[tokio::test]
async fn test_group_crud() {
    let mut conn = create_connection().await;

    clear_tables(&mut conn).await.unwrap();

    let group_name = "test_group_crud";
    let group_description = "group descr";

    let group1 = create_group(group_name, group_description);

    let group_entity = &mut DbEntity::from_data(group1, &conn).await.unwrap();

    group_entity.insert(&conn).await.unwrap();

    let find_result = Group::find_by_name(&conn, &group_name).await;
    assert!(
        find_result.is_ok(),
        format!("Failed to find group {}", &group_name)
    );
    assert!(
        find_result.unwrap().is_some(),
        format!("Group {} not found", &group_name)
    );

    assert!(group_entity.delete(&conn).await.is_ok(), "Delete Failed");

    let find_after_delete_result = Group::find_by_name(&conn, &group_name).await;

    assert!(
        find_after_delete_result.unwrap().is_none(),
        format!("Group {} not deleted", &group_name)
    );
}

#[tokio::test]
async fn test_user_group_join() {
    let mut conn = create_connection().await;
    clear_tables(&mut conn).await.unwrap();

    let user_name = "test_user_group_join-user1";
    let email = "test_user_group_join-user1@test.com";

    let user1 = create_user(user_name, email);

    let user_entity1 = &mut DbEntity::from_data(user1, &conn).await.unwrap();

    user_entity1.insert(&conn).await.unwrap();

    let group_name1 = "test_user_group_join-group1";
    let group_description1 = "group descr";

    let group1 = create_group(group_name1, group_description1);

    let mut group_entity1 = DbEntity::from_data(group1, &conn).await.unwrap();

    group_entity1.insert(&conn).await.unwrap();
    conn.execute(
        "INSERT INTO intrared.r_user_group(id_user, id_group) VALUES ($1, $2)",
        &[&user_entity1.id, &group_entity1.id],
    )
    .await
    .unwrap();

    let user1_groups = &mut user_groups(&user_entity1, &conn).await.unwrap();

    assert!(
        user1_groups.items.len() == 1,
        format!(
            "User should have just 1 associated group. {} groups where found.",
            user1_groups.items.len()
        )
    );
    let ug_grp1_entity = user1_groups.items.get(0).unwrap();
    assert_eq!(
        group_entity1.data.name, ug_grp1_entity.data.name,
        "Expected {}. Found {}",
        group_entity1.data.name, ug_grp1_entity.data.name
    );

    let group_name2 = "test_user_group_join-group2";
    let group_description2 = "group descr";

    let group2 = create_group(group_name2, group_description2);

    let mut group_entity2 = DbEntity::from_data(group2, &conn).await.unwrap();

    group_entity2.insert(&conn).await.unwrap();
    user1_groups.items.push(group_entity2);
    user1_groups.save_items(&mut conn).await.unwrap();
    user1_groups.sorting = vec!["data->>'name'".to_owned()];
    user1_groups.reload(&conn).await.unwrap();
    assert!(
        user1_groups.items.len() == 2,
        format!(
            "User should have just 2 associated groups. {} groups where found.",
            user1_groups.items.len()
        )
    );
    assert_eq!(
        user1_groups.items.get(0).unwrap().data.name,
        group_name1,
        "Expected user name {}, but {} was found.",
        group_name1,
        user1_groups.items.get(0).unwrap().data.name
    );
    user1_groups.sorting = vec!["data->>'name' DESC".to_owned()];
    user1_groups.reload(&conn).await.unwrap();
    assert_eq!(
        user1_groups.items.get(0).unwrap().data.name,
        group_name2,
        "Expected user name {}, but {} was found.",
        group_name2,
        user1_groups.items.get(0).unwrap().data.name
    );
}
