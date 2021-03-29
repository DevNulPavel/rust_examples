
/*struct FullUserData{
    data: UserInfo
}
impl actix_web::FromRequest for FullUserData{
    type Config = ();
    type Error = actix_web::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_http::Payload) -> Self::Future{
        let id = Identity::from_request(req, payload)
            .into_inner()
            .ok();

        let db = req.app_data::<web::Data<Database>>().unwrap().clone();

        // let req = req.clone();
        // let pl = payload;
        Box::pin(async move {
            if let Some(id) = id {
                if let Some(uuid) = id.identity(){
                    // Проверяем, что у нас валидный пользователь из базы
                    let info = db.try_find_full_user_info_for_uuid(&uuid).await?;
                    if let Some(info) = info {
                        return Ok(FullUserData{
                            data: info
                        });
                    }else{
                        // Сброс куки с идентификатором
                        id.forget();
            
                        return Err(actix_web::error::ErrorForbidden("Access forbidden"));                    
                    }
                }else{
                    return Err(actix_web::error::ErrorForbidden("Access forbidden"));
                }
            }else{
                return Err(actix_web::error::ErrorForbidden("Access forbidden"));
            }
        })
    }
}*/