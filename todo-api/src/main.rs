#[macro_use]
extern crate rocket;

use rocket::{ response::content, http::Status };
use serde::{ Serialize, Deserialize };
use rocket::serde::json::{ Json, to_string, from_str };
use redis::Commands;
use nanoid::nanoid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Todo {
    #[serde(default)]
    id: String,
    title: String,
    content: String,
    completed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct TodoCreatedResponse {
    message: String,
    id: String,
}

#[get("/")]
fn index() -> content::RawJson<&'static str> {
    // Add redis health cehck here
    content::RawJson("{\"message\": \"\"}")
}

#[get("/health")]
fn health() -> content::RawJson<&'static str> {
    dotenv::dotenv().ok();
    let redis_url = dotenv::var("REDIS_URL").expect("REDIS_URL must be set");
    let client: redis::Client = redis::Client::open(redis_url).unwrap();
    let mut con: redis::Connection = client.get_connection().unwrap();

    let pong: redis::RedisResult<()> = redis::cmd("PING").query(&mut con);

    match pong {
        Ok(_) => content::RawJson("{\"message\": \"Redis is running.\"}"),
        Err(_) => content::RawJson("{\"message\": \"Redis is not running.\"}"),
    }
}

#[get("/todos")]
fn get_todos() -> Json<Vec<Todo>> {
    let todos: Vec<Todo> = get_objects().expect("Failed to retrieve todos from Redis");

    Json(todos)
}

#[get("/todo/<id>")]
fn get_todo_by_id(id: String) -> Option<Json<Todo>> {
    let response: Todo = get_object(id.to_string()).expect("Failed to get object from Redis");

    Some(Json(response))
}

#[post("/todo", format = "json", data = "<todo>")]
fn add_todo(todo: Json<Todo>) -> Option<Json<TodoCreatedResponse>> {
    let new_todo: Todo = Todo {
        id: nanoid!(),
        title: todo.title.clone(),
        content: todo.content.clone(),
        completed: todo.completed,
    };

    set_object(new_todo.clone()).expect("Failed to store TODO in Redis");

    let created_todo_response: TodoCreatedResponse = TodoCreatedResponse {
        message: "Todo added successfully.".to_string(),
        id: new_todo.id,
    };
    Some(Json(created_todo_response))
}

#[put("/todo/<id>/complete")]
fn complete_todo(id: String) -> Result<content::RawJson<&'static str>, Status> {
    let mut todo: Todo = get_object(id.clone()).expect("Failed to get object from Redis");

    if todo.completed {
        return Err(Status::Conflict);
    }

    todo.completed = true;
    set_object(todo).expect("Failed to update TODO in Redis");

    Ok(content::RawJson("{\"message\": \"Todo marked as completed.\"}"))
}

#[catch(404)]
fn not_found() -> content::RawJson<&'static str> {
    content::RawJson("{\"message\": \"Resource not found.\"}")
}

#[catch(409)]
fn conflict() -> content::RawJson<&'static str> {
    content::RawJson("{\"message\": \"Todo already completed.\"}")
}

#[launch]
fn rocket() -> _ {
    rocket
        ::build()
        .mount("/", routes![index, get_todos, get_todo_by_id, add_todo, complete_todo, health])
        .register("/", catchers![not_found, conflict])
}

fn set_object(todo: Todo) -> redis::RedisResult<()> {
    dotenv::dotenv().ok();
    let redis_url = dotenv::var("REDIS_URL").expect("REDIS_URL must be set");
    let client: redis::Client = redis::Client::open(redis_url)?;
    let mut con: redis::Connection = client.get_connection()?;

    let todo_json: String = to_string(&todo).unwrap();

    let _: () = con.set(todo.id, todo_json)?;

    Ok(())
}

fn get_objects() -> redis::RedisResult<Vec<Todo>> {
    dotenv::dotenv().ok();
    let redis_url = dotenv::var("REDIS_URL").expect("REDIS_URL must be set");
    let client: redis::Client = redis::Client::open(redis_url)?;
    let mut con: redis::Connection = client.get_connection()?;

    let keys: Vec<String> = con.keys("*")?;

    let mut values: Vec<Todo> = Vec::new();
    for key in keys {
        let json: String = con.get(&key)?;
        let todo: Todo = from_str(&json).unwrap();
        values.push(todo);
    }

    Ok(values)
}

fn get_object(id: String) -> redis::RedisResult<Todo> {
    dotenv::dotenv().ok();
    let redis_url = dotenv::var("REDIS_URL").expect("REDIS_URL must be set");
    let client: redis::Client = redis::Client::open(redis_url)?;
    let mut con: redis::Connection = client.get_connection()?;
    let todo_json: Result<String, _> = con.get(id);

    match todo_json {
        Ok(json) => {
            let todo: Todo = from_str(&json).unwrap();
            println!("Todo: {:?}", todo);
            Ok(todo)
        }
        Err(_) => {
            // Handle the error here
            println!("Failed to get object from Redis: key may not exist or is not a string");
            Err(
                redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Operation against a key holding the wrong kind of value",
                ))
            )
        }
    }
}
