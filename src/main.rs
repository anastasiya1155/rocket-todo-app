mod schema;
mod models;

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

use std::io;

use rocket::tokio::task::spawn_blocking;
use rocket::tokio::time::{sleep, Duration};

use diesel::prelude::*;
use serde::{Deserialize};
use rocket::{fairing::AdHoc, serde::json::Json, State, response::Debug};
use rocket_sync_db_pools::database;

use models::Todo;

#[derive(Deserialize)]
struct Config {
    name: String,
    age: u8,
}

#[database("my_db")]
pub struct Db(diesel::PgConnection);

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}

#[get("/blocking_task")]
async fn blocking_task() -> io::Result<Vec<u8>> {
    // In a real app, use rocket::fs::NamedFile or tokio::fs::File.
    let vec = spawn_blocking(|| std::fs::read("data.txt")).await
        .map_err(|e| io::Error::new(io::ErrorKind::Interrupted, e))??;

    Ok(vec)
}

#[get("/<id>")]
async fn get_todo(
    connection: Db, id: i32
) -> Json<Todo> {
    connection
        .run(move |c| schema::todos::table.filter(schema::todos::id.eq(id)).first(c))
        .await
        .map(Json)
        .expect(format!("failed to fetch the todo with id: {}", id).as_str())
}

#[get("/")]
async fn get_all_todos(connection: Db) -> Json<Vec<Todo>> {
    connection
        .run(|c| schema::todos::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch todos")
}

#[post("/", data = "<todo>")]
async fn create_todo(connection: Db, todo: Json<Todo>) -> Json<Todo> {
    connection
        .run(move |c| {
            diesel::insert_into(schema::todos::table)
                .values(todo.into_inner())
                .get_result(c)
        })
        .await
        .map(Json)
        .expect("Failed to create todo")
}

#[get("/config")]
fn get_config(config: &State<Config>) -> String {
    format!(
        "Hello, {}! You are {} years old.",
        config.name, config.age
    )
}

#[delete("/<id>")]
async fn delete_todo(connection: Db, id: i32) -> Result<Option<()>, Debug<diesel::result::Error>> {
    let res = connection
        .run(move |c| {
            diesel::delete(schema::todos::table)
                .filter(schema::todos::id.eq(id))
                .execute(c)
        })
        .await?;

    Ok((res == 1).then(|| ()))
}

#[put("/<id>", data = "<todo>")]
async fn update_todo(connection: Db, id: i32, todo: Json<Todo>) -> Result<(), Debug<diesel::result::Error>> {
    connection
        .run(move |c| {
            diesel::update(schema::todos::table.filter(schema::todos::id.eq(id)))
                .set((
                    schema::todos::title.eq(&todo.title),
                    schema::todos::done.eq(&todo.done),
                ))
                .execute(c)
        }).await?;
    Ok(())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![world, delay, blocking_task, get_config])
        .mount("/todos", routes![get_todo, get_all_todos, create_todo, delete_todo, update_todo])
        .launch()
        .await?;

    Ok(())
}
