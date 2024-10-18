pub mod console_log;
pub use self::console_log::*;
use super::*;
mod ws_util;

use bytes::Buf;
use futures::{Stream, StreamExt};
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use std::convert::Infallible;
use std::net::SocketAddr;
use uuid::Uuid;
use warp::http::HeaderMap;
use warp::http::Method;
use warp::path::FullPath;
use warp::ws::{WebSocket, Ws};
use warp::Filter;

type HttpClient = hyper::Client<HttpsConnector<HttpConnector>>;

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Request {
    id: String,
    status: u16,
    is_replay: bool,
    path: String,
    query: Option<String>,
    method: Method,
    headers: HashMap<String, Vec<String>>,
    body_data: Vec<u8>,
    response_headers: HashMap<String, Vec<String>>,
    response_data: Vec<u8>,
    started: chrono::NaiveDateTime,
    completed: chrono::NaiveDateTime,
}

impl Request {
    pub fn path_and_query(&self) -> String {
        if let Some(query) = self.query.as_ref() {
            format!("{}?{}", self.path, query)
        } else {
            self.path.clone()
        }
    }
}

impl Request {
    pub fn elapsed(&self) -> String {
        let duration = self.completed - self.started;
        if duration.num_seconds() == 0 {
            format!("{}ms", duration.num_milliseconds())
        } else {
            format!("{}s", duration.num_seconds())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////

lazy_static::lazy_static! {
    // Хранилище запросов
    pub static ref REQUESTS: Arc<RwLock<HashMap<String, Request>>> = Arc::new(RwLock::new(HashMap::new()));
}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct IntrospectionAddrs {
    pub forward_address: SocketAddr,
    pub web_explorer_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ForwardError {
    IncomingRead,
    InvalidURL,
    InvalidRequest,
    LocalServerError,
}
impl warp::reject::Reject for ForwardError {

}

////////////////////////////////////////////////////////////////////////////////////////

/// Запускаем сервер ???
pub fn start_introspection_server(config: Config) -> IntrospectionAddrs {
    // Локальные адреса
    let local_addr = config.forward_url();
    let local_ws_addr = config.ws_forward_url();

    // Создаем http клиент
    let https = hyper_tls::HttpsConnector::new();
    let http_client = hyper::Client::builder()
        .build::<_, hyper::Body>(https);

    // Лямбда создания клиента
    let get_client = move || {
        let client = http_client.clone();
        warp::any()
            .map(move || {
                client.clone()
            })
            .boxed()
    };

    // Маршрут warp для входящих запросов, которые были проброшены
    let intercept = warp::any()
        // Локальный адрес
        .and(warp::any().map(move || {
            local_addr.clone()
        }))
        // Метод запроса
        .and(warp::method())
        // Полный путь
        .and(warp::path::full())
        // Сырой запрос
        .and(opt_raw_query())
        // Заголовки запроса
        .and(warp::header::headers_cloned())
        // Stream body
        .and(warp::body::stream())
        // http клиент
        .and(get_client())
        // Сам обработчик
        .and_then(forward);

    // Маршрут для WS данных, которые надо пробрасывать
    let intercept_ws = warp::any()
        // Прокидываем адрес, куда надо перенаправлять
        .and(warp::any().map(move || {
            local_ws_addr.clone()
        }))
        // Заголовок должен быть update
        .and(warp::header("upgrade"))
        // Пробрасываем метод
        .and(warp::method())
        // Заголовки
        .and(warp::header::headers_cloned())
        // Полный путь
        .and(warp::path::full())
        // Сырой запрос
        .and(opt_raw_query())
        // Непосредственно сам websocket
        .and(warp::ws())
        .map(move |addr: String, _upgrade: String, method: Method, headers: HeaderMap, 
                   path: FullPath, query: Option<String>, ws: Ws| {
                // Обработчик перехода в режим WS
                ws.on_upgrade(move |w: WebSocket| async {
                    forward_websocket(addr, path, method, headers, query, w).await
                })
            },
        );

    // Запускаем сервер для прерывания работы с возможностью прерывания
    // Возвращает адрес на котором запустится сервер и футуру
    let (forward_address, intercept_server) = warp::serve(intercept_ws.or(intercept))
        .bind_ephemeral(SocketAddr::from(([0, 0, 0, 0], 0)));
    tokio::spawn(intercept_server);

    // Обработчик css стилей
    let css = warp::get()
        .and(warp::path!("static" / "css" / "styles.css")
        .map(|| {
            let mut res = warp::http::Response::new(warp::hyper::Body::from(include_str!(
                "../../static/css/styles.css"
            )));
            res
                .headers_mut()
                .insert(warp::http::header::CONTENT_TYPE,
                        warp::http::header::HeaderValue::from_static("text/css"));
            res
        }));

    // Обработчик картинки логотипа
    let logo = warp::get()
        .and(warp::path!("static" / "img" / "logo.png")
        .map(|| {
            let mut res = warp::http::Response::new(warp::hyper::Body::from(
                include_bytes!("../../static/img/logo.png").to_vec(),
            ));
            res
                .headers_mut()
                .insert(warp::http::header::CONTENT_TYPE,
                        warp::http::header::HeaderValue::from_static("image/png"));
            res
        }));

    let forward_addr_clone = forward_address.clone();

    // Обработчик индекса
    let index = warp::get()
        .and(warp::path::end())
        .and_then(inspector);

    // Обработчик запроса деталей
    let detail = warp::get()
        .and(warp::path("detail"))
        .and(warp::path::param())
        .and_then(request_detail);

    // Обработчик повторного запроса
    let replay = warp::post()
        .and(warp::path("replay"))
        .and(warp::path::param())
        .and(get_client())
        .and_then(move |id, client| replay_request(id, client, forward_addr_clone.clone()));
    
    // Суммарный обработчик
    let web_explorer = index
        .or(detail)
        .or(replay)
        .or(css)
        .or(logo);

    // Запуск сервера проверки статуса
    let dash_addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], config.dashboard_port));
    let (web_explorer_address, explorer_server) = warp::serve(web_explorer)
        .bind_ephemeral(dash_addr);
    tokio::spawn(explorer_server);

    IntrospectionAddrs {
        forward_address,
        web_explorer_address,
    }
}

async fn forward(local_addr: String,
                 method: Method,
                 path: FullPath,
                 query: Option<String>,
                 headers: HeaderMap,
                 mut body: impl Stream<Item = Result<impl Buf, warp::Error>> + Send + Sync + Unpin + 'static,
                 client: HttpClient) -> Result<Box<dyn warp::Reply>, warp::reject::Rejection> {
    // Время запуска
    let started = chrono::Utc::now()
        .naive_utc();

    // Заголовки запроса
    let mut request_headers = HashMap::new();
    headers
        .keys()
        .for_each(|k| {
            let values = headers
                .get_all(k)
                .iter()
                .filter_map(|v| {
                    v
                        .to_str()
                        .ok()
                })
                .map(|s| {
                    s.to_owned()
                })
                .collect();
            request_headers
                .insert(k.as_str().to_owned(), values);
        });

    // Вычитываем все данные из тела
    // TODO: Может быть правильнее было бы перенаправлять body сразу, а не хранить в оперативке
    let mut request_body: Vec<u8> = vec![];
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(|e| {
            log::error!("error reading incoming buffer: {:?}", e);
            warp::reject::custom(ForwardError::IncomingRead)
        })?;

        request_body.extend_from_slice(chunk.chunk())
    }

    // Строка запроса
    let query_str = if let Some(query) = query.as_ref() {
        format!("?{}", query)
    } else {
        String::new()
    };

    // Конечный локальный URL
    let url = format!("{}{}{}", local_addr, path.as_str(), query_str);
    log::debug!("forwarding to: {}", &url);

    let uri = url
        .parse::<hyper::Uri>()
        .map_err(|e| {
            log::error!("invalid incoming url: {}, error: {:?}", url, e);
            warp::reject::custom(ForwardError::InvalidURL)
        })?;

    // Запрос
    let mut request = hyper::Request::builder()
        .method(method.clone())
        .version(hyper::Version::HTTP_11)
        .uri(uri);

    // Заголовки
    for header in headers {
        if let Some(header_name) = header.0 {
            request = request.header(header_name, header.1)
        }
    }

    // Запрос
    let request = request
        .body(hyper::Body::from(request_body.clone())) // TODO: Зачем сделано еще и клонирование?
        .map_err(|e| {
            log::error!("failed to build request: {:?}", e);
            warp::reject::custom(ForwardError::InvalidRequest)
        })?;

    // Ответ
    let response = client
        .request(request)
        .await
        .map_err(|e| {
            log::error!("local server error: {}", e);
            warp::reject::custom(ForwardError::LocalServerError)
        })?;

    // Заголовки ответа
    let mut response_headers = HashMap::new();
    response
        .headers()
        .keys()
        .for_each(|k| {
            let values = response
                .headers()
                .get_all(k)
                .iter()
                .filter_map(|v| {
                    v
                        .to_str()
                        .ok()
                })
                .map(|s| {
                    s.to_owned()
                })
                .collect();
            response_headers
                .insert(k.as_str().to_owned(), values);
        });

    let (parts, mut body) = response.into_parts();

    // Вычитываем данные ответа в оперативку
    // TODO: На больших запросах оперативки может не хватить
    let mut response_data = vec![];
    while let Some(next) = body.data().await {
        let chunk = next
            .map_err(|e| {
                log::error!("error reading local response: {:?}", e);
                warp::reject::custom(ForwardError::LocalServerError)
            })?;

        response_data
            .extend_from_slice(&chunk);
    }

    // Формируем объект с данными запроса
    let stored_request = Request {
        id: Uuid::new_v4().to_string(),
        status: parts.status.as_u16(),
        path: path.as_str().to_owned(),
        query,
        method,
        headers: request_headers,
        body_data: request_body,              // TODO: Надо ли сохранять?
        response_headers,
        response_data: response_data.clone(), // TODO: Надо ли сохранять?
        started,
        completed: chrono::Utc::now().naive_utc(),
        is_replay: false,
    };

    // Сохраняем в хранилище
    REQUESTS
        .write()
        .unwrap()
        .insert(stored_request.id.clone(), stored_request);

    // Выдаем ответ
    Ok(Box::new(warp::http::Response::from_parts(
        parts,
        response_data,
    )))
}

/// Прокидывание запросов вебсокетов
async fn forward_websocket(local_addr: String,
                           path: FullPath,
                           method: Method,
                           headers: HeaderMap,
                           query: Option<String>,
                           websocket: WebSocket) {
    log::debug!("connecting to websocket");

    // Строка запроса
    let query_str = if let Some(query) = query.as_ref() {
        format!("?{}", query)
    } else {
        String::new()
    };

    // Урл, куда пробрасываем
    let url = format!("{}{}{}", local_addr, path.as_str(), query_str);
    log::debug!("forwarding to: {}", &url);

    // id запроса
    let request_id = Uuid::new_v4();

    // Перегоняем все заголовки запроса
    let mut request_headers = HashMap::new();
    headers
        .keys()
        .for_each(|k| {
            let values = headers
                .get_all(k)
                .iter()
                .filter_map(|v| {
                    v.to_str().ok()
                })
                .map(|s| {
                    s.to_owned()
                })
                .collect();
            request_headers.insert(k.as_str().to_owned(), values);
        });

    // Объект запроса
    let stored_request = Request {
        id: request_id.to_string(),
        status: 101,
        path: path.as_str().to_owned(),
        query,
        method,
        headers: request_headers,
        body_data: b"Websocket Data".to_vec(),
        response_headers: Default::default(),
        response_data: vec![],
        started: chrono::Utc::now().naive_utc(),
        completed: chrono::Utc::now().naive_utc(),
        is_replay: false,
    };

    // Сохраняем запрос в список
    REQUESTS
        .write()
        .unwrap()
        .insert(request_id.to_string(), stored_request);

    // Непосредственно проброс запроса
    let _ = forward_websocket_inner(request_id, url, websocket)
        .await
        .map_err(|e| {
            error!("websocket error occurred: {:?}", e);
        });
}

async fn forward_websocket_inner(request_id: Uuid,
                                 url: String,
                                 incoming: WebSocket) -> Result<(), Box<dyn std::error::Error>> {
    // Локально подключаемся к локальному url
    let (local, _) = tokio_tungstenite::connect_async(&url)
        .await?;
    // Разделение на входящий и исходящий потоки у локального адреса
    let (mut local_send, mut local_receive) = local.split();
    // Входящий поток данных снаружи
    let (mut incoming_send, mut incoming_receive) = incoming.split();

    // incoming_r -> local_w
    // Запуск обработки входящего внешнего во входящий внутренний поток
    tokio::spawn(async move {
        // Читаем в цикле
        while let Some(Ok(next)) = incoming_receive.next().await {
            // Отладочный вывод данных
            let debug_data = vec!["\n\nIncoming data => ".as_bytes(), next.as_bytes()].concat();

            // Конвертируем сообщение из warp библиотеки в tung
            let message = match ws_util::warp_to_tung(next) {
                Ok(m) => m,
                Err(e) => {
                    error!("invalid ws message: {:?}", e);
                    continue;
                }
            };

            // Отсылаем локальному серверу
            if let Err(e) = local_send.send(message.clone()).await {
                error!("failed to write to local websocket: {:?}", e);
                break;
            }

            // Для нашего сохраненного запроса расширяем отгруженные данные
            REQUESTS
                .write()
                .unwrap()
                .get_mut(&request_id.to_string())
                .map(|req| {
                    req.response_data.extend_from_slice(&debug_data);
                });
        }
    });

    // Отправляем данные из внутреннего сервера во внешний получатель
    tokio::spawn(async move {
        while let Some(Ok(next)) = local_receive.next().await {
            // Конвертация сообщения одной библиотеки в другую
            let message = match ws_util::tung_to_warp(next) {
                Ok(m) => m,
                Err(e) => {
                    error!("invalid ws message: {:?}", e);
                    continue;
                }
            };

            // Сообщение для отладки
            let debug_data = vec!["\n\nOutgoing data => ".as_bytes(), message.as_bytes()].concat();

            // Выгружаем
            if let Err(e) = incoming_send.send(message).await {
                error!("failed to write to incoming websocket: {:?}", e);
                break;
            }

            // В сохраненном запросе расширяем данные нашей отгрузкой
            REQUESTS
                .write()
                .unwrap()
                .get_mut(&request_id.to_string())
                .map(|req| {
                    req.response_data.extend_from_slice(&debug_data);
                });
        }
    });

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, askama::Template)]
#[template(path = "index.html")]
struct Inspector {
    requests: Vec<Request>,
}

#[derive(Debug, Clone, askama::Template)]
#[template(path = "detail.html")]
struct InspectorDetail {
    request: Request,
    incoming: BodyData,
    response: BodyData,
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
struct BodyData {
    data_type: DataType,
    content: Option<String>,
    raw: String,
}

impl AsRef<BodyData> for BodyData {
    fn as_ref(&self) -> &BodyData {
        &self
    }
}

#[derive(Debug, Clone)]
enum DataType {
    Json,
    Unknown,
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Обработчик индекса
async fn inspector() -> Result<Page<Inspector>, warp::reject::Rejection> {
    // Делаем копии всех запросов
    // TODO: Может не делать копии? Просто использовать Arc?
    let mut requests: Vec<Request> = REQUESTS
        .read()
        .unwrap()
        .values()
        .map(|r| {
            r.clone()
        })
        .collect();
    
    // Сортировка по дате завершения
    requests.sort_by(|a, b| {
        b.completed.cmp(&a.completed)
    });

    // Генерируем страничку
    let inspect = Inspector { requests };

    Ok(Page(inspect))
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Запрос деталей конкретного запроса
async fn request_detail(rid: String) -> Result<Page<InspectorDetail>, warp::reject::Rejection> {
    let request: Request = match REQUESTS.read().unwrap().get(&rid) {
        Some(r) => {
            r.clone()
        },
        None => {
            return Err(warp::reject::not_found());
        },
    };

    let detail = InspectorDetail {
        incoming: get_body_data(&request.body_data),
        response: get_body_data(&request.response_data),
        request,
    };

    Ok(Page(detail))
}

fn get_body_data(input: &[u8]) -> BodyData {
    let mut body = BodyData {
        data_type: DataType::Unknown,
        content: None,
        raw: std::str::from_utf8(input)
                .map(|s| {
                    s.to_string()
                })
                .unwrap_or("No UTF-8 Data".to_string()),
    };

    // Пытаемся распарсить как json
    match serde_json::from_slice::<serde_json::Value>(input) {
        Ok(serde_json::Value::Object(map)) => {
            body.data_type = DataType::Json;
            body.content = serde_json::to_string_pretty(&map).ok();
        }
        Ok(serde_json::Value::Array(arr)) => {
            body.data_type = DataType::Json;
            body.content = serde_json::to_string_pretty(&arr).ok();
        }
        _ => {}
    }

    body
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Обработчик повторения запроса
async fn replay_request(rid: String,
                        client: HttpClient,
                        addr: SocketAddr) -> Result<Box<dyn warp::Reply>, warp::reject::Rejection> {
    // Находим запрос
    let request: Request = match REQUESTS.read().unwrap().get(&rid) {
        Some(r) => r.clone(),
        None => return Err(warp::reject::not_found()),
    };

    // Строка запроса
    let query_str = if let Some(query) = request.query.as_ref() {
        format!("?{}", query)
    } else {
        String::new()
    };

    // Полный адрес внутренний
    let url = format!("http://localhost:{}{}{}",
        addr.port(),
        &request.path,
        query_str
    );

    let uri = url
        .parse::<hyper::Uri>()
        .map_err(|e| {
            log::error!("invalid incoming url: {}, error: {:?}", url, e);
            warp::reject::custom(ForwardError::InvalidURL)
        })?;

    // Сам запрос
    let mut new_request = hyper::Request::builder()
        .method(request.method)
        .version(hyper::Version::HTTP_11)
        .uri(uri);

    // Заголовки запроса
    for (header, values) in &request.headers {
        for v in values {
            new_request = new_request.header(header, v)
        }
    }

    // Body
    let new_request = new_request
        .body(hyper::Body::from(request.body_data))
        .map_err(|e| {
            log::error!("failed to build request: {:?}", e);
            warp::reject::custom(ForwardError::InvalidRequest)
        })?;

    // Выполняем запрос
    let _ = client.request(new_request).await.map_err(|e| {
        log::error!("local server error: {:?}", e);
        warp::reject::custom(ForwardError::LocalServerError)
    })?;

    // Отвечаем необходимостью перехода в корень
    let response = warp::http::Response::builder()
        .status(warp::http::StatusCode::SEE_OTHER)
        .header(warp::http::header::LOCATION, "/")
        .body(b"".to_vec());

    Ok(Box::new(response))
}

/////////////////////////////////////////////////////////////////////////////

// Тип страницы
struct Page<T>(T);

impl<T> warp::reply::Reply for Page<T>
where
    T: askama::Template + Send + 'static,
{
    fn into_response(self) -> warp::reply::Response {
        let res = self.0.render().unwrap();

        warp::http::Response::builder()
            .status(warp::http::StatusCode::OK)
            .header(warp::http::header::CONTENT_TYPE, "text/html")
            .body(res.into())
            .unwrap()
    }
}

/////////////////////////////////////////////////////////////////////////////

fn opt_raw_query() -> impl Filter<Extract = (Option<String>,), Error = Infallible> + Copy {
    warp::filters::query::raw()
        .map(|q| Some(q))
        .or(warp::any().map(|| None))
        .unify()
}
