

// This struct represents state
#[derive(Debug)]
pub struct AppState {
    pub app_name: String,
    pub counter: std::sync::Mutex<i32>
}

impl AppState {
    pub fn new(name: &str) -> AppState{
        AppState{
            app_name: String::from(name),
            counter: std::sync::Mutex::new(0)
        }
    }
}