use actix_web::{
    dev::{
        ServiceRequest
    },
    FromRequest
};
use actix_identity::{
    Identity
};

// #[instrument]
/*pub async fn get_uuid_from_ident_with_db_check(id: &Identity,
                                               db: &web::Data<Database>) -> Result<Option<String>, AppError>{
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как 
    // есть проблемы с асинхронным запросом к базе в middleware
    if let Some(uuid) = id.identity() {
        // Проверяем, что у нас валидный пользователь из базы
        let exists = db.does_user_uuid_exist(&uuid).await?;
        if !exists {
            // Сброс куки с идентификатором
            id.forget();

            return Ok(None);
        }else{
            return Ok(Some(uuid));
        }
    }else{
        return Ok(None);
    }
}*/

// #[instrument]
/*pub async fn get_full_user_info_for_identity(id: &Identity,
                                             db: &web::Data<Database>) -> Result<Option<UserInfo>, AppError>{
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как 
    // есть проблемы с асинхронным запросом к базе в middleware
    if let Some(uuid) = id.identity(){
        // Проверяем, что у нас валидный пользователь из базы
        let info = db.try_find_full_user_info_for_uuid(&uuid).await?;
        if info.is_none() {
            // Сброс куки с идентификатором
            id.forget();

            return Ok(None);
        }else{
            return Ok(info);
        }
    }else{
        return Ok(None);
    }
}*/

pub fn get_user_id_from_request(req: ServiceRequest) -> (Option<String>, ServiceRequest) {
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