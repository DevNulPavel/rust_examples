// use diesel::{
//     prelude::{
//         *
//     },
//     table
// };

table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}
