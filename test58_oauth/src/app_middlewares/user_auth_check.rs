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
        Database
    },
    helpers::{
        get_user_id_from_request
    }
};

////////////////////////////////////////////////////////////////////////////

pub fn create_auth_check_middleware<F>(if_exists: bool, fail_callback: F) -> AuthCheck
where
    F: Fn() -> HttpResponse + 'static
{
    AuthCheck{
        if_exists,
        fail_callback: Arc::new(fail_callback)
    }
}

////////////////////////////////////////////////////////////////////////////

// Структура нашего конвертера в миддлевару
pub struct AuthCheck{
    if_exists: bool,
    fail_callback: Arc<dyn Fn() -> HttpResponse> // TODO: в шаблоны?
}

// Реализация трансформа для перехода в Middleware
impl<S, B> Transform<S> for AuthCheck
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
    type Transform = AuthCheckMiddleware<S>;       // Во что трансформируемся
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Создаем новую middleware из параметра, обязательно сохраняем наш сервис там
        let middleware = AuthCheckMiddleware{ 
            service: Rc::new(RefCell::new(service)),
            if_exists: self.if_exists,
            fail_callback: self.fail_callback.clone()
        };
        futures::future::ok(middleware)
    }
}

////////////////////////////////////////////////////////////////////////////

/// Непосредственно сам Middleware, хранящий следующий сервис
pub struct AuthCheckMiddleware<S> {
    // TODO: Убрать Mutex? Заменить на async Mutex
    // https://github.com/casbin-rs/actix-casbin-auth/blob/master/src/middleware.rs
    service: Rc<RefCell<S>>,
    if_exists: bool,
    fail_callback: Arc<dyn Fn() -> HttpResponse> // TODO: в шаблоны?
}

/// Реализация работы Middleware сервиса
impl<S, B> Service for AuthCheckMiddleware<S>
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
        // let mut guard = self.service.lock().expect("AuthCheckMiddleware lock failed");
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // Делаем клоны Arc для передачи в лямбду
        let mut srv = self.service.clone();
        let fail_callback = self.fail_callback.clone();
        let if_exists = self.if_exists;

        // Получаем user_id из запроса
        let (user_uuid, req) = get_user_id_from_request(req);

        // Получаем базу данных из запроса
        let db = req
            .app_data::<web::Data<Database>>()
            .expect("Database manager is missing")
            .clone();

        Box::pin(async move {
            // Дергаем данные
            let user_exists = match user_uuid {
                Some(user_uuid) => {
                    db.does_user_uuid_exist(&user_uuid).await?
                },
                None => {
                    false
                }
            };

            let user_exists = if if_exists {
                user_exists
            }else{
                !user_exists
            };

            // Ищем полную информацию о пользователе
            if user_exists {
                // TODO: Проверить на сколько блокируется
                // let mut guard = srv.lock().expect("CheckLoginMiddleware lock failed");
                srv.call(req).await
            }else{
                // Переходим куда надо если нет данных о пользователе
                let http_redirect_resp = fail_callback().into_body();
                Ok(req.into_response(http_redirect_resp))
            }
        })
    }
}
