use quick_error::{
    quick_error
};

quick_error!{
    #[derive(Debug)]
    pub enum FondyError {
        RequestError(err: reqwest::Error){
            from()
        }

        JsonParseError(err: serde_json::Error){
            from()
        }

        InternalError{
        }
    }
}



