use quick_error::{
    quick_error
};

quick_error! {
    #[derive(Debug)]
    pub enum AppError {
        /// Rabbit error
        ZMQ(err: zeromq::ZmqError) {
            from()
        }
    }
}