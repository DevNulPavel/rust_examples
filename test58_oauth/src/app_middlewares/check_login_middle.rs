// TODO: Вариант проверки логина без middleware: https://www.youtube.com/channel/UCdZV0XJvls773ljUEYTsNzA/videos
// https://stackoverflow.com/questions/62269278/how-can-i-make-protected-routes-in-actix-web
// https://github.com/DDtKey/actix-web-grants/blob/main/src/middleware.rs
// https://stackoverflow.com/a/63158225

use std::{
    task::{
        Context, 
        Poll
    },
    pin::{
        Pin
    },
    future::{
        Future
    },
    sync::{
        Arc,
        Mutex
    }
};
use actix_http::{
    HttpMessage
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

// Структура нашего конвертера в миддлевару
#[derive(Default)]
pub struct CheckLogin{
}

// Реализация трансформа для перехода в Middleware
impl<S, B> Transform<S> for CheckLogin
where
    S: Service<Request = ServiceRequest, 
               Response = ServiceResponse<B>, 
               Error = Error> + 'static,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Создаем новую middleware из параметра, обязательно сохраняем наш сервис там
        let middleware = CheckLoginMiddleware{ 
            service: Arc::new(Mutex::new(service))
        };
        futures::future::ok(middleware)
    }
}

////////////////////////////////////////////////////////////////////////////

/// непосредственно сам Middleware, хранящий следующий сервис
pub struct CheckLoginMiddleware<S> {
    // TODO: Убрать Mutex?
    // https://github.com/casbin-rs/actix-casbin-auth/blob/master/src/middleware.rs
    service: Arc<Mutex<S>>,
}

/// Реализация работы Middleware сервиса
impl<S, B> Service for CheckLoginMiddleware<S>
where
    S: Service<Request = ServiceRequest, 
               Response = ServiceResponse<B>, 
               Error = Error> + 'static,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        // TODO: Проверить на сколько блокируется
        let mut guard = self.service.lock().expect("CheckLoginMiddleware lock failed");
        guard.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
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

            let user_info = if let Some(user_id) = user_id{
                db.try_find_full_user_info_for_uuid(&user_id).await?
            }else{
                None
            };

            if let Some(user_info) = user_info {                
                // Если мы залогинены, тогда не пускаем на страницу логина
                if req.path() == constants::LOGIN_PATH {
                    // Переходим на главную
                    let http_redirect_resp = HttpResponse::Found()
                        .header(http::header::LOCATION, constants::INDEX_PATH)
                        .finish()
                        .into_body();
        
                    Ok(req.into_response(http_redirect_resp))
                }else{
                    // Сохраняем расширение для передачи в виде параметра в метод
                    req.extensions_mut().insert(user_info);

                    // TODO: Проверить на сколько блокируется
                    let mut guard = srv.lock().expect("CheckLoginMiddleware lock failed");
                    guard.call(req).await
                }
            } else {                
                // Если мы не залогинены, проверяем - может мы уже на странице логина?
                if req.path() != constants::LOGIN_PATH {
                    // Переходим на логин
                    let http_redirect_resp = HttpResponse::Found()
                        .header(http::header::LOCATION, constants::LOGIN_PATH)
                        .finish()
                        .into_body();
        
                    Ok(req.into_response(http_redirect_resp))
                }else{
                    // Сохраняем расширение для передачи в виде параметра в метод
                    req.extensions_mut().insert(user_info);

                    // TODO: Проверить на сколько блокируется
                    let mut guard = srv.lock().expect("CheckLoginMiddleware lock failed");
                    guard.call(req).await
                }
            }
        })
    }
}
