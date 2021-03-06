YAQB (This will not be the real name. Please open PRs with name ideas)
======================================================================

[![Build Status](https://travis-ci.org/sgrif/yaqb.svg)](https://travis-ci.org/sgrif/yaqb)
[Documentation](http://sgrif.github.io/yaqb/yaqb/index.html)

This is an early stage ORM in Rust. It is poorly documented, and rapidly
iterating. I would love early feedback on usage. Help in documenting current
usage would also be welcomed.

The goal is to take a different approach here. This is not a port of Active
Record or Hibernate. This is an attempt to find what a "Rust ORM" is. So far,
what that seems to be is something that is statically guaranteed to only allow
correct queries, while still feeling high level.

An "incorrect query" includes, but is not limited to:

- Invalid SQL syntax
- Attempting to interpret a column as the wrong type (e.g. reading varchar as
  i32, treating a nullable column as something other than an option)
- Selecting a column from another table
- Selecting columns that are not used (this doesn't mean that you have to access
  that field on your struct, but the struct must have that field)

Does it support X?
------------------

0.1 progress is tracked on https://github.com/sgrif/yaqb/issues/1

Getting Started
---------------

Before you can do anything, you'll first need to set up your table. You'll want
to specify the columns and tables that exist using the [`table!` macro][table]
Once you've done that, you can already start using the query builder, and
pulling out primitives.

Much of the behavior in yaqb comes from traits, and it is recommended that you
import `yaqb::*`. We avoid exporting generic type names, or any bare functions
at that level.

```rust
#[macro_use]
extern crate yaqb;

use yaqb::*;

table! {
    users {
        id -> Serial,
        name -> VarChar,
        favorite_color -> Nullable<VarChar>,
    }
}

fn users_with_name(connection: &Connection, target_name: &str)
    -> Vec<(i32, String, Option<String>)>
{
    use self::users::dsl::*;
    users.filter(name.eq(target_name))
        .load(connection)
        .unwrap()
        .collect()
}
```

Note that we're importing `users::dsl::*` here. This allows us to deal with
only the users table, and not have to qualify everything. If we did not have
this import, we'd need to put `users::` before each column, and reference the
table as `users::table`.

If you want to be able to query for a struct, you'll need to implement the
[`Queriable` trait][queriable] Luckily,
[yaqb_codegen](https://github.com/sgrif/yaqb/tree/master/yaqb_codegen) can do
this for us automatically.

```rust
#[derive(Queriable, Debug)]
pub struct User {
    id: i32,
    name: String,
    favorite_color: Option<String>,
}

fn main() {
    let connection = Connection::establish(env!("DATABASE_URL"))
        .unwrap();
    let users: Vec<User> = users::table.load(&connection)
        .unwrap().collect();

    println!("Here are all the users in our database: {:?}", users);
}
```

Insert
------

Inserting data requires implementing the [`Insertable` trait][insertable] Once
again, we can have this be automatically implemented for us by the compiler.

```rust
#[insertable_into(users)]
struct NewUser<'a> {
    name: &'a str,
    favorite_color: Option<&'a str>,
}

fn create_user(connection: &Connection, name: &str, favorite_color: Option<&str>)
  -> QueryResult<User>
{
    let new_user = NewUser {
        name: name,
        favorite_color: favorite_color,
    };
    connection.insert(&users::table, &new_user)
        .map(|mut result| result.nth(0).unwrap())
}
```

[`insert`][insert] can return any struct which implements
[`Queriable`][queriable] for the right type. If you don't actually want to use
the results, you should call [`insert_returning_count`][insert_returning_count]
instead, or the compiler will complain that it can't infer what type you meant
to return. You can use the same struct for inserting and querying if you'd like,
but you'll need to make columns that are not present during the insert optional
(e.g. `id` and timestamps). For this reason, you probably want to create a new
struct intead.

You might notice that we're having to manually grab the first record that was
inserted. This is because [`insert`][insert] can also take a slice or `Vec` of
records, and will insert them in a single query. For this reason,
[`insert`][insert] will always return an `Iterator`. A helper for this common
case will likely be added in the future.

For both `#[derive(Queriable)]` and `#[insertable_into]`, you can annotate any
single field with `#[column_name="name"]`, if the name of your field differs
from the name of the column. This annotation is required on all fields of tuple
structs. This cannot be used, however, to work around name collisions with
keywords that are reserved in Rust, as you cannot have a column with that name.
This may change in the future.

```rust
#[insertable_into(users)]
struct NewUser<'a>(
    #[column_name="name"]
    &'a str,
    #[column_name="favorite_color"]
    Option<&'a str>,
)

fn create_user(connection: &Connection, name: &str, favorite_color: Option<&str>)
  -> QueryResult<User>
{
    let new_user = NewUser(name, favorite_color);
    connection.insert(&users::table, &new_user)
        .map(|mut result| result.nth(0).unwrap())
}
```

Update
------

To update a record, you'll need to call the [`update`][update] function. Unlike
[`insert`][insert] (which may change to use this pattern in the future),
[`update`][update] is a top level function which creates a query that you'll
later pass to the [`Connection`][connection]. Here's a simple example.

```rust
fn change_users_name(connection: &Connection, target: i32, new_name: &str) -> QueryResult<User> {
    use yaqb::query_builder::update;
    use users::dsl::*;

    let command = update(users.filter(id.eq(target))).set(name.eq(new_name));
    connection.query_one(command)
        .map(|r| r.unwrap())
}
```

Similar to [`insert`][insert], we always return a `Result<Option<Model>>`, as we can't
tell at compile time if this is the kind of query that always returns at least 1
result. This may change in the future.

As with [`insert`][insert], we can return any type which implements
[`Queriable`][queriable] for the right types. If you do not want to use the
returned record(s), you should call
[`execute_returning_count`][execute_returning_count] instead of
[`query_one`][query_one] or [`query_all`][query_all].

You can also use a struct to represent the changes, if it implements
[`AsChangeset`][as_changeset]. Again, `yaqb_codegen` can generate this for us
automatically.

```rust
#[changeset_for(users)]
pub struct UserChanges {
    name: String,
    favorite_color: Option<String>,
}

fn save_user(connection: &Connection, id: i32, changes: &UserChanges) -> QueryResult<Option<User>> {
    let command = update(users::table.filter(users::id.eq(id))).set(changes);
    connection.query_one(command)
}
```

Note that even though we've implemented [`AsChangeset`][as_changeset], we still
need to specify what records we want to update. If the struct has the primary
key on it, a method called `save_changes` will also be added.

```rust
#[changeset_for(users)]
pub struct User {
    id: i32,
    name: String,
    favorite_color: Option<String>,
}

fn change_name_to_jim(connection: &Connection, user: &mut User) -> QueryResult<()> {
    user.name = "Jim".into();
    user.save_changes(connection)
}
```

This method will update the model with any fields that are updated in the
database (for example, if you have timestamps which are updated by triggers).

Delete
------

[`delete`][delete] works very similarly to [`update`][delete], but does not
support returning a record.

```rust
fn delete_user(connection: &Connection, user: User) -> QueryResult<()> {
    use yaqb::query_builder::delete;
    use users::dsl::*;

    let command = delete(users.filter(id.eq(user.id)));
    let deleted_rows = try!(connection.execute_returning_count(&command));
    debug_assert!(deleted_rows == 1);
    Ok(())
}
```

How do I do other things?
-------------------------

Take a look at the various files named on what you're trying to do in
https://github.com/sgrif/yaqb/tree/master/yaqb_tests/tests. See
https://github.com/sgrif/yaqb/blob/master/yaqb_tests/tests/schema.rs for how
you can go about getting the data structures set up.

[table]: http://sgrif.github.io/yaqb/yaqb/macro.table!.html
[queriable]: http://sgrif.github.io/yaqb/yaqb/query_source/trait.Queriable.html
[insertable]: http://sgrif.github.io/yaqb/yaqb/persistable/trait.Insertable.html
[insert]: http://sgrif.github.io/yaqb/yaqb/struct.Connection.html#method.insert
[insert_returning_count]: http://sgrif.github.io/yaqb/yaqb/struct.Connection.html#method.insert_returning_count
[execute_returning_count]: http://sgrif.github.io/yaqb/yaqb/struct.Connection.html#method.execute_returning_count
[query_one]: http://sgrif.github.io/yaqb/yaqb/struct.Connection.html#method.query_one
[query_all]: http://sgrif.github.io/yaqb/yaqb/struct.Connection.html#method.query_all
[update]: http://sgrif.github.io/yaqb/yaqb/query_builder/fn.update.html
[delete]: http://sgrif.github.io/yaqb/yaqb/query_builder/fn.delete.html
[connection]: http://sgrif.github.io/yaqb/yaqb/struct.Connection.html
[as_changeset]: http://sgrif.github.io/yaqb/yaqb/query_builder/update_statement/changeset/trait.AsChangeset.html
