use std::{
    sync::{
        Arc
    }
};
use tracing::{
    debug,
    error,
    instrument,
};
use tracing_subscriber::{
    prelude::{
        *
    },
    fmt::{
        format::{
            FmtSpan
        }
    }
};
use warp::{
    Filter,
    Reply,
    Rejection,
    reject::{
        Reject
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    json
};
use tap::{
    prelude::{
        *
    }
};
use crate::{
    error::{
        FondyError
    },
    application::{
        Application
    }
};


impl Reject for FondyError {
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(app))]
async fn index(app: Arc<Application>) -> Result<impl Reply, Rejection>{
    let html = app
        .templates
        .render("index", &json!({}))
        .map_err(FondyError::from)
        .tap_err(|err| { error!("Index template rendering failed: {}", err); })?;

    Ok(warp::reply::html(html))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct BuyItemParams{
    item_id: i32
}

#[instrument(skip(app))]
async fn buy(app: Arc<Application>, buy_params: BuyItemParams) -> Result<impl Reply, Rejection>{
    debug!("Buy params: {:#?}", buy_params);

    Ok(warp::redirect(warp::http::Uri::from_static("/")))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument]
async fn rejection_to_json(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = rejection.find::<FondyError>(){
        let reply = warp::reply::json(&json!({
            "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": err.to_string()
        }));
        Ok(warp::reply::with_status(reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
    }else{
        Err(rejection)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

pub async fn start_server(app: Arc<Application>) {
    let index_app = app.clone();
    let index = warp::path::end()
        .and(warp::get())    
        .and(warp::any().map(move || { 
            index_app.clone()
        }))
        .and_then(index);

    let buy_app = app.clone();
    let buy = warp::path::path("buy")
        .and(warp::post())
        .and(warp::any().map(move || { 
            buy_app.clone()
        }))
        .and(warp::filters::body::form())
        .and_then(buy)
        .recover(rejection_to_json);

    let routes = index
        .or(buy);

    warp::serve(routes)
        .bind(([0, 0, 0, 0], 8080))
        .await;
}