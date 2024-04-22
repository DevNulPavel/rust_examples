// TODO: Можно ли как-то избавиться?
#[macro_use]
extern crate diesel;

mod schema;
mod models;

use std::{
    env::{
        self
    }
};
use diesel::{
    prelude::{
        *
    },
    pg::{
        PgConnection
    }
};
use models::Post;

fn create_new_post(db_conn: &PgConnection, new_post: models::NewPost) -> models::Post {
    let post = diesel::insert_into(schema::posts::table)
        .values(&new_post)
        .get_result(db_conn)
        .expect("Insert post failed");
    post
}

fn publish_post(db_conn: &PgConnection, post_id: i32) {
    diesel::update(schema::posts::dsl::posts.find(post_id))
        .set(schema::posts::dsl::published.eq_all(true))
        .execute(db_conn)
        .expect("Publish post failed");
}

fn delete_post_by_title_pattern(db_conn: &PgConnection, pattern: &str) {
    use schema::posts::dsl::*;
    diesel::delete(posts.filter(title.like(format!("%{}%", pattern))))
        .execute(db_conn)
        .expect("Posts deleted");
}

fn main() {
    // Читаем файлик .env и добавляем переменные из файлика в окружение
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL variable is missing");
    let db_conn = PgConnection::establish(&&db_url).expect("Database connection error");

    // Новый пост
    let post_1 = create_new_post(&db_conn, models::NewPost{
        title: "Title 1",
        body: "Body 1"
    });
    println!("New post: {:#?}", post_1);

    // Новый пост
    let post_2 = create_new_post(&db_conn, models::NewPost{
        title: "Title 2",
        body: "Body 2"
    });
    println!("New post: {:#?}", post_2);

    // Публикация постов
    publish_post(&db_conn, post_2.id);

    // Удаление поста по условию
    delete_post_by_title_pattern(&db_conn, "Title 1");

    // Мы можем выводить непосредственно SQL запроса
    let debug_query = schema::posts::dsl::posts.limit(10);
    let debug_query_str = diesel::debug_query::<diesel::pg::Pg, _>(&debug_query);
    println!("Debug query: {:}", debug_query_str);

    // Получаем посты из базы
    let results = schema::posts::dsl::posts
        .filter(schema::posts::dsl::published.eq_all(true))
        .limit(5)
        .load::<models::Post>(&db_conn)
        .expect("Posts loading failed");

    println!("Published posts: {} len", results.len());

    results
        .into_iter()
        .for_each(|val|{
            println!("Post: {:#?}", val);
        });
}