use super::schema::*;
use yaqb::*;

#[test]
fn insert_records() {
    use schema::users::table as users;
    let connection = connection();
    setup_users_table(&connection);

    let new_users: &[_] = &[
        NewUser::new("Sean", Some("Black")),
        NewUser::new("Tess", None),
    ];
    let inserted_users: Vec<_> = connection.insert(&users, new_users).unwrap().collect();

    let expected_users = vec![
        User { id: 1, name: "Sean".to_string(), hair_color: Some("Black".to_string()) },
        User { id: 2, name: "Tess".to_string(), hair_color: None },
    ];
    let actual_users: Vec<_> = users.load(&connection).unwrap().collect();

    assert_eq!(expected_users, actual_users);
    assert_eq!(expected_users, inserted_users);
}

#[test]
fn insert_with_defaults() {
    use schema::users::table as users;
    let connection = connection();
    connection.execute("CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR NOT NULL DEFAULT 'Green'
    )").unwrap();
    let new_users: &[_] = &[
        NewUser::new("Sean", Some("Black")),
        NewUser::new("Tess", None),
    ];
    let inserted_users: Vec<_> = connection.insert(&users, new_users).unwrap().collect();

    let expected_users = vec![
        User { id: 1, name: "Sean".to_string(), hair_color: Some("Black".to_string()) },
        User { id: 2, name: "Tess".to_string(), hair_color: Some("Green".to_string()) },
    ];
    let actual_users: Vec<_> = users.load(&connection).unwrap().collect();

    assert_eq!(expected_users, actual_users);
    assert_eq!(expected_users, inserted_users);
}

#[test]
fn insert_with_defaults_not_provided() {
    use schema::users::table as users;
    let connection = connection();
    connection.execute("CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR NOT NULL DEFAULT 'Green'
    )").unwrap();
    let new_users: &[_] = &[
        BaldUser { name: "Sean".to_string() },
        BaldUser { name: "Tess".to_string() },
    ];
    let inserted_users: Vec<_> = connection.insert(&users, new_users).unwrap().collect();

    let expected_users = vec![
        User { id: 1, name: "Sean".to_string(), hair_color: Some("Green".to_string()) },
        User { id: 2, name: "Tess".to_string(), hair_color: Some("Green".to_string()) },
    ];
    let actual_users: Vec<_> = users.load(&connection).unwrap().collect();

    assert_eq!(expected_users, actual_users);
    assert_eq!(expected_users, inserted_users);
}

#[test]
fn insert_returning_count_returns_number_of_rows_inserted() {
    use schema::users::table as users;
    let connection = connection();
    connection.execute("CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR NOT NULL DEFAULT 'Green'
    )").unwrap();
    let new_users: &[_] = &[
        BaldUser { name: "Sean".to_string() },
        BaldUser { name: "Tess".to_string() },
    ];
    let count = connection.insert_returning_count(&users, new_users).unwrap();
    let second_count = connection.insert_returning_count(&users, &BaldUser { name: "Guy".to_string() }).unwrap();

    assert_eq!(2, count);
    assert_eq!(1, second_count);
}

#[insertable_into(users)]
struct BaldUser {
    name: String,
}

#[insertable_into(users)]
struct BorrowedUser<'a> {
    name: &'a str,
}

#[test]
fn insert_borrowed_content() {
    use schema::users::table as users;
    let connection = connection();
    setup_users_table(&connection);
    let new_users: &[_] = &[
        BorrowedUser { name: "Sean" },
        BorrowedUser { name: "Tess" },
    ];
    let inserted_users: Vec<_> = connection.insert(&users, new_users).unwrap().collect();

    let expected_users = vec![
        User::new(1, "Sean"),
        User::new(2, "Tess"),
    ];
    let actual_users: Vec<_> = users.load(&connection).unwrap().collect();

    assert_eq!(expected_users, actual_users);
    assert_eq!(expected_users, inserted_users);
}

#[test]
fn delete() {
    use schema::users::dsl::*;
    use yaqb::query_builder::delete;
    let connection = connection_with_sean_and_tess_in_users_table();

    let command = delete(users.filter(name.eq("Sean")));
    let deleted_rows = connection.execute_returning_count(&command).unwrap();

    assert_eq!(1, deleted_rows);

    let num_users = users.count().first(&connection).unwrap();

    assert_eq!(Some(1), num_users);
}
