// Импортируем результат работы Proto - имя пакета
pub mod routeguide {
    tonic::include_proto!("route_guide");
}

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use routeguide::route_guide_server::{RouteGuide, RouteGuideServer}; // route_guide_server::RouteGuideServer генерируется protobuf
use routeguide::{Feature, Point, Rectangle, RouteNote, RouteSummary};

mod load_data{
    use serde::Deserialize;
    use std::fs::File;

    #[derive(Debug, Deserialize)]
    struct Feature {
        location: Location,
        name: String,
    }
    
    #[derive(Debug, Deserialize)]
    struct Location {
        latitude: i32,
        longitude: i32,
    }
    
    #[allow(dead_code)]
    pub fn load() -> Vec<super::Feature> {
        let file = File::open("data/route_data.json").expect("failed to open data file");
    
        let decoded: Vec<Feature> =
            serde_json::from_reader(&file).expect("failed to deserialize features");
    
        decoded
            .into_iter()
            .map(|feature| super::Feature {
                name: feature.name,
                location: Some(super::Point {
                    longitude: feature.location.longitude,
                    latitude: feature.location.latitude,
                }),
            })
            .collect()
    }
}


// Реализация хэширования для точки
impl Hash for Point {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.latitude.hash(state);
        self.longitude.hash(state);
    }
}

impl Eq for Point {
}

fn in_range(point: &Point, rect: &Rectangle) -> bool {
    use std::cmp;

    let lo = rect.lo.as_ref().unwrap();
    let hi = rect.hi.as_ref().unwrap();

    let left = cmp::min(lo.longitude, hi.longitude);
    let right = cmp::max(lo.longitude, hi.longitude);
    let top = cmp::max(lo.latitude, hi.latitude);
    let bottom = cmp::min(lo.latitude, hi.latitude);

    point.longitude >= left
        && point.longitude <= right
        && point.latitude >= bottom
        && point.latitude <= top
}

/// Calculates the distance between two points using the "haversine" formula.
/// This code was taken from http://www.movable-type.co.uk/scripts/latlong.html.
fn calc_distance(p1: &Point, p2: &Point) -> i32 {
    const CORD_FACTOR: f64 = 1e7;
    const R: f64 = 6_371_000.0; // meters

    let lat1 = p1.latitude as f64 / CORD_FACTOR;
    let lat2 = p2.latitude as f64 / CORD_FACTOR;
    let lng1 = p1.longitude as f64 / CORD_FACTOR;
    let lng2 = p2.longitude as f64 / CORD_FACTOR;

    let lat_rad1 = lat1.to_radians();
    let lat_rad2 = lat2.to_radians();

    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lng = (lng2 - lng1).to_radians();

    let a = (delta_lat / 2f64).sin() * (delta_lat / 2f64).sin()
        + (lat_rad1).cos() * (lat_rad2).cos() * (delta_lng / 2f64).sin() * (delta_lng / 2f64).sin();

    let c = 2f64 * a.sqrt().atan2((1f64 - a).sqrt());

    (R * c) as i32
}

// Описываем класс сервера
#[derive(Debug)]
pub struct RouteGuideService {
    features: Arc<Vec<Feature>>, // TODO: Нужна ли синхронизация с помощью MUTEX
}

// Реализация трейта-обработчика
#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    // Поток особенностей, выдаваемых клиенту - это Receiver из Tokio канала
    type ListFeaturesStream = mpsc::Receiver<Result<Feature, Status>>;
    // Канал из входных данных в выходные - Box для потока из futures, который припинен к конкретному месту в памяти
    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + Sync + 'static>>;

    // Вызов получения особенности, конверируется из GetFeature
    async fn get_feature(&self, request: Request<Point>) -> Result<Response<Feature>, Status> {
        println!("GetFeature = {:?}", request);

        // Обходим все варианты
        for feature in self.features.iter() { // Можно создать слайс вот так &self.features[..]
            // Если расположение совпадает - возвращаем результат
            if feature.location.as_ref() == Some(request.get_ref()) {
                return Ok(Response::new(feature.clone()));
            }
        }

        // Если нет - пустой вариаант
        Ok(Response::new(Feature::default()))
    }

    // Обработка получения списка особенностей в области
    async fn list_features(&self, request: Request<Rectangle>) -> Result<Response<Self::ListFeaturesStream>, Status> {
        println!("ListFeatures = {:?}", request);

        // Создаем канал Tokio
        let (mut sender, receiver) = mpsc::channel(4);
        
        // Потокобезопасный указатель на список фич
        let features = self.features.clone();

        // Создаем новую задачу в Tokio, которая будет постепенно спамить данные в канал
        tokio::spawn(async move {
            // Обходим все фичи
            for feature in features.iter() { // Можно создать слайс вот так &self.features[..]
                // Проверяем принадлежность ректанглу
                if in_range(feature.location.as_ref().unwrap(), request.get_ref()) {
                    println!("  => send {:?}", feature);
                    // Отправляем в канал данные с ожиданием получения
                    sender.send(Ok(feature.clone()))
                        .await
                        .unwrap();
                }
            }

            println!(" /// done sending");
            // Здесь уничтожается передатчик канала
        });

        // Возвращаем Tokio канал
        Ok(Response::new(receiver))
    }

    // Запись данных в наш сервис
    async fn record_route(&self, request: Request<tonic::Streaming<Point>>) -> Result<Response<RouteSummary>, Status> {
        println!("RecordRoute");

        // Параметр - это поток данных
        let mut stream = request.into_inner();

        // Результат - путь
        let mut summary = RouteSummary::default();
        let mut last_point = None;
        
        // Текущее время
        let now = Instant::now();

        // Получаем данные из потока
        while let Some(point) = stream.next().await {
            // Валидная ли точка?
            let point = point?;

            println!("  ==> Point = {:?}", point);

            // Увеличиваем количество точек
            summary.point_count += 1;

            // Ищем особенности для данной точки 
            for feature in &self.features[..] {
                if feature.location.as_ref() == Some(&point) {
                    summary.feature_count += 1;
                }
            }

            // Вычисляем дистанцию
            if let Some(ref last_point) = last_point {
                summary.distance += calc_distance(last_point, &point);
            }

            last_point = Some(point);
        }

        summary.elapsed_time = now.elapsed().as_secs() as i32;

        Ok(Response::new(summary))
    }

    async fn route_chat(&self, request: Request<tonic::Streaming<RouteNote>>) -> Result<Response<Self::RouteChatStream>, Status> {
        println!("RouteChat");

        let mut notes = HashMap::new();

        // Входной поток данных
        let mut stream = request.into_inner();

        // Выходной поток данных
        let output = async_stream::try_stream! {
            // Получаем данные из входного потока
            while let Some(note) = stream.next().await {
                let note = note?;

                let location = note.location.clone().unwrap();

                let location_notes = notes.entry(location).or_insert(vec![]);
                location_notes.push(note);

                // Для каждого расположения - выбрасываем новые данные
                for note in location_notes {
                    yield note.clone();
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::RouteChatStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:10000".parse().unwrap(); // V6 - [::1]:10000

    println!("RouteGuideServer listening on: {}", addr);

    let data = load_data::load();

    // Создаем наш сервис
    let route_guide = RouteGuideService {
        features: Arc::new(data),
    };

    // Создаем непосредственно сервер
    let svc = RouteGuideServer::new(route_guide);

    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}