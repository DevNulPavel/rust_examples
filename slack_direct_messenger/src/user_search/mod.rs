// Файлик, содержащие корнневое описание

// Сами по себе модули не экспортируем, только функции из них
mod by_email;
mod by_name;

pub use by_email::find_user_id_by_email;
pub use by_name::find_user_id_by_name;
