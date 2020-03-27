/// Реализация *Actor* Message трейта для конкретного типа сообщения и его ответа
macro_rules! simple_req_resp_impl {
    ($type: ty, $result_type: ty) => {
        
        impl<A, M> ::actix::dev::MessageResponse<A, M> for $result_type
        where
            A: ::actix::dev::Actor,
            M: ::actix::dev::Message<Result = $result_type>
        {
            fn handle<R: ::actix::dev::ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
                if let Some(tx) = tx {
                    tx.send(self);
                }
            }
        }

        impl ::actix::dev::Message for $type {
            type Result = $result_type;
        }
    };
}

/// Реализует Actix *Web* HTTP Responder для простого типа, который может быть превращен в JSON
macro_rules! json_responder_impl {
    ($type: ty) => {
        impl ::actix_web::Responder for $type {
            type Item = ::actix_web::HttpResponse;
            type Error = ::actix_web::Error;

            fn respond_to<S>(self, _: &::actix_web::HttpRequest<S>) -> Result<::actix_web::HttpResponse,::actix_web::Error> {
                // Create response and set content type
                Ok(::actix_web::HttpResponse::Ok().json(self))
            }
        }
    };
}
