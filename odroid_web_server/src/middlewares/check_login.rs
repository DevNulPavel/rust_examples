use std::{
    task::{
        Context, 
        Poll
    }
};
use actix_web::{
    dev::{
        ServiceRequest, 
        ServiceResponse
    },
    http, 
    Error, 
    HttpResponse, 
    FromRequest
};
use actix_service::{
    Service, 
    Transform
};
use actix_identity::{
    Identity
};
use futures::{
    future::{
        ok, 
        Either, 
        Ready
    }
};
use crate::{
    constants
};


// Структура нашего враппера
#[derive(Default)]
pub struct CheckLogin{
}

// Реализация трансформа для перехода в Middleware
impl<S, B> Transform<S> for CheckLogin
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Создаем новую middleware из параметра
        let middleware = CheckLoginMiddleware{ 
            service 
        };
        futures::future::ok(middleware)
    }
}

////////////////////////////////////////////////////////////////////////////

/// непосредственно сам Middleware, хранящий следующий сервис
pub struct CheckLoginMiddleware<S> {
    service: S,
}

/// Реализация работы Middleware сервиса
impl<S, B> Service for CheckLoginMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // Нам нужно лижшь подключиться к `start` для этого middleware

        let (r, mut pl) = req.into_parts(); 
        let is_logged_in = {
            // https://github.com/actix/actix-web/issues/1499#issuecomment-626967053
            match Identity::from_request(&r, &mut pl).into_inner() {
                Ok(identity) => {
                    println!("Identity exists");
                    match identity.identity(){
                        Some(user) => {
                            println!("Identity user {}", user);
                            if user == constants::LOGIN{
                                true
                            }else{
                                false
                            }
                        },
                        None => {
                            println!("Identity user is None");
                            false
                        }
                    }
                },
                Err(_) => {
                    println!("Identity error");
                    false
                }
            }
        };
        let req = ServiceRequest::from_parts(r, pl)
            .ok()
            .unwrap();

        // Если мы залогинены, тогда выполняем запрос как обычно
        if is_logged_in {
            println!("Is logged in");
            // Если мы залогинены, тогда не пускаем на страницу логина
            if req.path() == "/login" {
                // Переходим на главную
                Either::Right(ok(req.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, "/index")
                        .finish()
                        .into_body(),
                )))
            }else{
                Either::Left(self.service.call(req))
            }
        } else {
            println!("Is NOT logged in");

            // Если мы не залогинены, проверяем - может мы уже на странице логина?
            if req.path() == "/login" {
                Either::Left(self.service.call(req))
            } else {
                // Если не на странице логина - переходим туда
                Either::Right(ok(req.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, "/login")
                        .finish()
                        .into_body(),
                )))
            }
        }
    }
}