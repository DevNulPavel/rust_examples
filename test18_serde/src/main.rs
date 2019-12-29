use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

fn default_val() -> i32{
    4
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]  // Аттрибут контейнера, запрещает неизвестные поля при парсинге
#[serde(rename_all = "snake_case")]
struct Point {
    #[serde(rename(serialize = "xx", deserialize = "xx"))]  // Используем стандартное имя
    x: i32,
    #[serde(rename = "a")] // Используем другое имя
    y: i32,
    #[serde(default)] // Значение должно использоваться дефолтное, если поля нету
    z: i32,
    #[serde(alias = "ww")] // Можно задавать алиас, тогда парсинг будет происходить по стандартному или по альтернативному имени
    #[serde(default = "default_val")]  // Дефолтное значение будет получено из метода
    w: i32,
    #[serde(skip)] // #[serde(skip_serializing)], #[serde(skip_deserializing)] Пропускает значение при сериализации
    meta: i32
}

// Теги позоляют делать следующее - в зависимости от значения типа выбирать как выполнять сериализацию и тд
// {"type": "Request", "id": "", "method": "", "params": ""}
// {"type": "Response", "id": "", "result": "" }
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    Request { id: String, method: String, params: String },
    Response { id: String, result: String },
}

 // Такой вариант позволяет разделить указание типа и содержимое
// {"t": "Para", "c": [2, 3]}
// {"t": "Str", "c": "the string"}
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Block {
    Para(Vec<i32>),
    Str(String),
}

// Либо можно не указывать теги, тогда serde попытается самостоятельно попытаться распарсить значения
// {"id": "", "method": "", "params": {}}
// {"id": "", "result": ""}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum MessageUntagged {
    Request { id: String, method: String, params: String },
    Response { id: String, result: String },
}

// Можно разворачивать структуру в уровень выше
// {
//     "limit": 100,
//     "offset": 200,
//     "total": 1053,
//     "users": [
//       {"id": "49824073-979f-4814-be10-5ea416ee1c2f", "username": "john_doe"},
//       ...
//     ]
// }
#[derive(Serialize, Deserialize)]
struct Pagination {
    limit: u64,
    offset: u64,
    total: u64,
}
#[derive(Serialize, Deserialize)]
struct Users {
    users: Vec<User>,

    #[serde(flatten)]
    pagination: Pagination,
}

// можно разворачивать неизвестные значения в значания Value, которые можно потом использовать
// { "id": "", "username": "", "mascot": "Ferris" }
#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    username: String,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

fn main() {
    let point = Point { x: 1, y: 2, z: 2, w: 1, meta: 2 };

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized = {:?}", deserialized);
}