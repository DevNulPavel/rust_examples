use crate::http::jwt::json_web_token::Identity;
use crate::http::shared::{AppState, RequestDetails};
use axum::body::Body;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

const AUTHORIZATION: &str = "authorization";
const BEARER: &str = "Bearer ";
const UNAUTHORIZED: StatusCode = StatusCode::UNAUTHORIZED;

const UNAUTHORIZED_PATHS: &[&str] = &[
    "/",
    "/metrics",
    "/ping",
    "/users/login",
    "/users/refresh-token",
    "/personal-access-tokens/login",
];

pub async fn jwt_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if UNAUTHORIZED_PATHS.contains(&request.uri().path()) {
        return Ok(next.run(request).await);
    }

    let bearer = request
        .headers()
        .get(AUTHORIZATION)
        .ok_or(UNAUTHORIZED)?
        .to_str()
        .map_err(|_| UNAUTHORIZED)?;

    if !bearer.starts_with(BEARER) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let jwt_token = &bearer[BEARER.len()..];
    let token_header = jsonwebtoken::decode_header(jwt_token).map_err(|_| UNAUTHORIZED)?;
    let jwt_claims = state
        .jwt_manager
        .decode(jwt_token, token_header.alg)
        .map_err(|_| UNAUTHORIZED)?;
    if state
        .jwt_manager
        .is_token_revoked(&jwt_claims.claims.jti)
        .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let request_details = request.extensions().get::<RequestDetails>().unwrap();
    let identity = Identity {
        token_id: jwt_claims.claims.jti,
        token_expiry: jwt_claims.claims.exp,
        user_id: jwt_claims.claims.sub,
        ip_address: request_details.ip_address,
    };
    request.extensions_mut().insert(identity);
    Ok(next.run(request).await)
}
