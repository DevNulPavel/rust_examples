/*use std::{
    task::{
        Context, 
        Poll
    }
};
use actix_web::{
    web::{
        self
    },
    dev::{
        ServiceRequest, 
        ServiceResponse
    }, 
    http::{
        self
    },
    Error, 
    FromRequest, 
    HttpResponse
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
use log::{
    debug
};
use crate::{
    constants::{
        self
    },
    database::{
        Database
    }
};

////////////////////////////////////////////////////////////////////////////

pub fn create_check_login_middleware() -> CheckLogin{
    CheckLogin::default()
}

////////////////////////////////////////////////////////////////////////////

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

        // debug!("Check path {}", req.path());

        let (r, mut pl) = req.into_parts(); 
        let is_logged_in = {
            // https://github.com/actix/actix-web/issues/1499#issuecomment-626967053
            // https://github.com/actix/actix-web/issues/1517
            // https://www.reddit.com/r/rust/comments/gvaash/stuck_with_actix_and_supporting_redirect_to_login/fsnke8c/
            match Identity::from_request(&r, &mut pl).into_inner() {
                Ok(identity) => {
                    // println!("Identity exists");
                    identity.identity().is_some()
                },
                Err(_) => {
                    // println!("Identity error");
                    false
                }
            }
        };
        let req = ServiceRequest::from_parts(r, pl)
            .ok()
            .unwrap();

        // Если мы залогинены, тогда выполняем запрос как обычно
        if is_logged_in {
            // debug!("Is logged in");

            // Если мы залогинены, тогда не пускаем на страницу логина
            if req.path() == constants::LOGIN_PATH {
                // Переходим на главную
                Either::Right(ok(req.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, constants::INDEX_PATH)
                        .finish()
                        .into_body(),
                )))
            }else{
                Either::Left(self.service.call(req))
            }
        } else {
            // debug!("Is NOT logged in");

            // Если мы не залогинены, проверяем - может мы уже на странице логина?
            if req.path() == constants::LOGIN_PATH {
                Either::Left(self.service.call(req))
            } else {
                // Если не на странице логина - переходим туда
                Either::Right(ok(req.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, constants::LOGIN_PATH)
                        .finish()
                        .into_body(),
                )))
            }
        }
    }
}*/







/*.wrap_fn(|req, srv| {
                    use actix_service::{
                        Service
                    };
                    use futures::{
                        future::{
                            ok, 
                            Either, 
                            Ready
                        }
                    };
                    use actix_web::{web, App, FromRequest, HttpResponse, http, dev::ServiceResponse};
                    use actix_web::http::{header::CONTENT_TYPE, HeaderValue};

                    let http_req = req.into_parts().0.clone()
                    let f = srv.call(req);
                    let red = ServiceResponse::new(http_req, HttpResponse::Found()
                                                                .header(http::header::LOCATION, "/")
                                                                .finish()
                                                                .into_body());

                    async move {
                        let (r, mut pl) = req.into_parts();
                        let user_id = Identity::from_request(&r, &mut pl)
                            .into_inner()
                            .ok()
                            .and_then(|identity|{
                                identity.identity()
                            });
                        let req = actix_web::dev::ServiceRequest::from_parts(r, pl)
                            .ok()
                            .unwrap();
    
                        let db = req
                            .app_data::<web::Data<Database>>()
                            .expect("Database manager is missing")
                            .clone();

                        let has_user = if let Some(user_id) = user_id{
                            db.try_find_user_with_uuid(&user_id).await.is_ok()
                        }else{
                            false
                        };
    
                        if has_user {                
                            // Если мы залогинены, тогда не пускаем на страницу логина
                            if req.path() == constants::LOGIN_PATH {
                                // srv_local.call(req).await
                                // Переходим на главную
                                Ok(red)
                            }else{
                                f.await
                            }
                        } else {                
                            // Если мы не залогинены, проверяем - может мы уже на странице логина?
                            if req.path() != constants::LOGIN_PATH {
                                // srv_local.call(req).await
                                // Переходим на главную
                                Ok(req.into_response(HttpResponse::Found()
                                                        .header(http::header::LOCATION, "/login")
                                                        .finish()
                                                        .into_body()))
                            }else{
                                f.await
                            }
                        }
                    }

                    /*async {
                        let (r, mut pl) = req.into_parts(); 

                        let req = actix_web::dev::ServiceRequest::from_parts(r, pl)
                            .ok()
                            .unwrap();
                    }*/

                    

                    /*async {
                        let mut res = fut.await?;
                        res.headers_mut().insert(
                           CONTENT_TYPE, HeaderValue::from_static("text/plain"),
                        );
                        Ok(res)
                    }*/
                })*/
                /*.wrap_fn(|req, srv| {
                    use actix_service::{
                        Service
                    };
                    use futures::{
                        future::{
                            ok, 
                            Either, 
                            Ready
                        }
                    };
                    use actix_web::{web, App, FromRequest, HttpResponse, http, dev::ServiceResponse};
                    use actix_web::http::{header::CONTENT_TYPE, HeaderValue};

                    Box::pin(async move {
                        srv.call(req).await
                    })
                })*/