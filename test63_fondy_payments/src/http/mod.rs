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
    }
};


impl warp::reject::Reject for FondyError {
}


#[instrument]
async fn index() -> Result<impl Reply, Rejection>{
    Err(FondyError::InternalError)
        .tap_err(|err| { error!("Server error: {}", err); })?;

    Ok(warp::reply::html("asds"))
}

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

pub async fn start_server() {
    let routes = warp::path("test")
        .and(warp::get())
        .and_then(index);

    warp::serve(routes)
        .bind(([0, 0, 0, 0], 8080))
        .await;
}