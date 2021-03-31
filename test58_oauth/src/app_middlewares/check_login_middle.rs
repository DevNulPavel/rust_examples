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
use tracing::{
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

pub fn create_check_login_middleware<P, F>(patch_check: P,
                                           path_fail: F) -> CheckLogin
where
    P: Fn(&str) -> bool + 'static,
    F: Fn() -> HttpResponse + 'static
{
    CheckLogin{
        path_check_fn: Arc::new(patch_check),
        path_fail_response_fn: Arc::new(path_fail)
    }
}

////////////////////////////////////////////////////////////////////////////

// Структура нашего конвертера в миддлевару
pub struct CheckLogin{
    pub path_check_fn: Arc<dyn Fn(&str) -> bool>, // TODO: в шаблоны?
    pub path_fail_response_fn: Arc<dyn Fn() -> HttpResponse> // TODO: в шаблоны?
}

// Реализация трансформа для перехода в Middleware
impl<S, B> Transform<S> for CheckLogin
where
    S: Service<Request = ServiceRequest, 
               Response = ServiceResponse<B>, 
               Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;       // Во что трансформируемся
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Создаем новую middleware из параметра, обязательно сохраняем наш сервис там
        let middleware = CheckLoginMiddleware{ 
            service: Arc::new(Mutex::new(service)),
            path_check_fn: self.path_check_fn.clone(),
            path_fail_response_fn: self.path_fail_response_fn.clone()
        };
        futures::future::ok(middleware)
    }
}

////////////////////////////////////////////////////////////////////////////

fn get_user_id_from_request(req: ServiceRequest) -> (Option<String>, ServiceRequest) {
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
    (user_id, req)
}

/// Непосредственно сам Middleware, хранящий следующий сервис
pub struct CheckLoginMiddleware<S> {
    // TODO: Убрать Mutex?
    // https://github.com/casbin-rs/actix-casbin-auth/blob/master/src/middleware.rs
    service: Arc<Mutex<S>>,
    path_check_fn: Arc<dyn Fn(&str) -> bool>, // TODO: в шаблоны?
    path_fail_response_fn: Arc<dyn Fn() -> HttpResponse> // TODO: в шаблоны?
}

/// Реализация работы Middleware сервиса
impl<S, B> Service for CheckLoginMiddleware<S>
where
    S: Service<Request = ServiceRequest, 
               Response = ServiceResponse<B>, 
               Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        // TODO: Проверить на сколько блокируется
        let mut guard = self.service.lock().expect("CheckLoginMiddleware lock failed");
        guard.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let path_check_fn = self.path_check_fn.clone();
        let path_fail_response_fn = self.path_fail_response_fn.clone();

        Box::pin(async move {
            // Получаем user_id из запроса
            let (user_id, req) = get_user_id_from_request(req);

            // Получаем базу данных из запроса
            let db = req
                .app_data::<web::Data<Database>>()
                .expect("Database manager is missing")
                .clone();

            let user_info = match user_id {
                Some(user_id) => db.try_find_full_user_info_for_uuid(&user_id).await?,
                None => None
            };

            // Ищем полную информацию о пользователе
            match user_info {
                Some(user_info) if path_check_fn(req.path()) => {
                    // Сохраняем расширение для передачи в виде параметра в метод
                    req.extensions_mut().insert(user_info);

                    // TODO: Проверить на сколько блокируется
                    let mut guard = srv.lock().expect("CheckLoginMiddleware lock failed");
                    guard.call(req).await
                },
                _ => {
                    // Переходим куда надо если нет данных о пользователе
                    let http_redirect_resp = path_fail_response_fn().into_body();
                    Ok(req.into_response(http_redirect_resp))
                }
            }
        })
    }
}
