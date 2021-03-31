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
        Arc
    },
    rc::{
        Rc
    },
    cell::{
        RefCell
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
    HttpResponse
};
use actix_service::{
    Service, 
    Transform
};
use futures::{
    future::{
        Ready
    }
};
// use tracing::{
//     debug
// };
use crate::{
    database::{
        Database,
        UserInfo
    },
    helpers::{
        get_user_id_from_request
    }
};

////////////////////////////////////////////////////////////////////////////

impl actix_web::FromRequest for UserInfo {
    type Config = ();
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future{
        let ext = req.extensions();
        match ext.get::<UserInfo>() {
            Some(full_info) => {
                futures::future::ready(Ok(full_info.clone())) // TODO: Убрать клон?
            },
            None => {
                futures::future::ready(Err(actix_web::error::ErrorUnauthorized("User info is missing")))
            }
        }        
    }
}

////////////////////////////////////////////////////////////////////////////

pub fn create_user_info_middleware<F>(no_user_response_fn: F) -> UserInfoProvider
where
    F: Fn() -> HttpResponse + 'static
{
    UserInfoProvider{
        no_user_response_fn: Arc::new(no_user_response_fn)
    }
}

////////////////////////////////////////////////////////////////////////////

// Структура нашего конвертера в миддлевару
pub struct UserInfoProvider{
    pub no_user_response_fn: Arc<dyn Fn() -> HttpResponse> // TODO: в шаблоны?
}

// Реализация трансформа для перехода в Middleware
impl<S, B> Transform<S> for UserInfoProvider
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
    type Transform = UserInfoMiddleware<S>;       // Во что трансформируемся
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Создаем новую middleware из параметра, обязательно сохраняем наш сервис там
        let middleware = UserInfoMiddleware{ 
            service: Rc::new(RefCell::new(service)),
            no_user_response_fn: self.no_user_response_fn.clone()
        };
        futures::future::ok(middleware)
    }
}

////////////////////////////////////////////////////////////////////////////

/// Непосредственно сам Middleware, хранящий следующий сервис
pub struct UserInfoMiddleware<S> {
    // TODO: Убрать Mutex? Заменить на async Mutex?
    // https://github.com/casbin-rs/actix-casbin-auth/blob/master/src/middleware.rs
    // service: Arc<Mutex<S>>,
    service: Rc<RefCell<S>>,
    no_user_response_fn: Arc<dyn Fn() -> HttpResponse> // TODO: в шаблоны?
}

/// Реализация работы Middleware сервиса
impl<S, B> Service for UserInfoMiddleware<S>
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
        // let mut guard = self.service.lock().expect("CheckLoginMiddleware lock failed");
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // Делаем клоны Arc для передачи в лямбду
        let mut srv = self.service.clone();
        let no_user_response_fn = self.no_user_response_fn.clone();

        // Получаем user_id из запроса
        let (user_id, req) = get_user_id_from_request(req);

        // Получаем базу данных из запроса
        let db = req
            .app_data::<web::Data<Database>>()
            .expect("Database manager is missing")
            .clone();

        Box::pin(async move {
            // Дергаем данные
            let user_info = match user_id {
                Some(user_id) => db.try_find_full_user_info_for_uuid(&user_id).await?,
                None => None
            };

            // Ищем полную информацию о пользователе
            match user_info {
                Some(user_info) => {
                    // Сохраняем расширение для передачи в виде параметра в метод
                    req.extensions_mut().insert(user_info);

                    // TODO: Проверить на сколько блокируется
                    // let mut guard = srv.lock().expect("CheckLoginMiddleware lock failed");
                    // guard.call(req).await
                    srv.call(req).await
                },
                None => {
                    // Переходим куда надо если нет данных о пользователе
                    let http_redirect_resp = no_user_response_fn().into_body();
                    Ok(req.into_response(http_redirect_resp))
                }
            }
        })
    }
}
