use std::{
    io::{
        self,
        Write
    }
};

/// Способ запросить у пользователя определенный ввод с клавиатуры
pub fn prompt_user(prompt: &str) -> String {
    warning!(prompt);
    io::stdout()
        .flush()
        .expect("Couldn't flush stdout!");

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .ok()
        .expect("Couldn't read line!");

    // Remove w+
    String::from(user_input.trim())
}