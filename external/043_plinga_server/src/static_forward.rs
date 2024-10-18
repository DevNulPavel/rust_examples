/*use url::{
    Url
};
use log::{
    debug
};
use actix_web::{
    web::{
        self
    },
    client::{
        Client
    },
    HttpResponse
};

pub async fn static_forward(req: web::HttpRequest, 
                            body: web::Bytes,
                            target_url: web::Data<Url>,
                            client: web::Data<Client>) -> Result<HttpResponse, actix_web::Error> {
    // Создаем новый URL
    let new_url = {
        let mut new_url = target_url.get_ref().clone();
        new_url.set_path(req.path());
        new_url.set_query(req.uri().query());
        new_url
    };

    debug!("Redirect url: {}", new_url);

    // Создаем новый запрос к серверу
    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    // Выполняем запрос
    let mut res = forwarded_req
        .send_body(body)
        .await
        .map_err(actix_web::Error::from)?;

    let mut client_resp = HttpResponse::build(res.status());

    // Отбрасываем заголовок соединения
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    let headers_iter = res
        .headers()
        .iter()
        .filter(|(h, _)| {
            *h != "connection"
        });
    for (header_name, header_value) in headers_iter {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp
        .body(res.body()
        .await?))
}*/