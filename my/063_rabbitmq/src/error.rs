use quick_error::{
    quick_error
};

quick_error! {
    #[derive(Debug)]
    pub enum RabbitError {
        /// Rabbit error
        Io(err: lapin::Error) {
            from()
        }
    }
}