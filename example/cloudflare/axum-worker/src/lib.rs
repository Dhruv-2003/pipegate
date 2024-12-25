use alloy::primitives::U256;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::{self, Next},
    routing::{get, post},
    Router,
};

use pipegate::{
    channel::ChannelState,
    utils::{modify_headers_axum, parse_headers_axum},
    verify::verify_and_update_channel,
};
use tower_service::Service;
use worker::*;

#[derive(Clone)]
pub(crate) struct AppState {
    channel_state: ChannelState,
    env: Env,
}

fn router(env: Env) -> Router {
    let state = ChannelState::new("https://base-sepolia-rpc.publicnode.com".parse().unwrap());

    let app_state = AppState {
        channel_state: state.clone(),
        env,
    };

    Router::new()
        .route("/", get(root))
        .route("/generate", post(generate))
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state,
            auth_middleware::<Body>,
        ))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router(env).call(req).await?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}

async fn auth_middleware<B>(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let payment_amount = U256::from(1000);

    let state = state.channel_state;
    // Parse headers
    match parse_headers_axum(req, payment_amount).await {
        Ok((signed_request, parts)) => {
            let body_bytes = signed_request.body_bytes.clone();
            match verify_and_update_channel(&state, signed_request).await {
                Ok(updated_channel) => {
                    // Reconstruct request with original body
                    let req = Request::from_parts(parts, Body::from(body_bytes));
                    let response = next.run(req).await;
                    modify_headers_axum(response, &updated_channel)
                }
                Err(e) => Response::builder().status(e).body(Body::empty()).unwrap(),
            }
        }
        Err(e) => Response::builder().status(e).body(Body::empty()).unwrap(),
    }
}

async fn generate(State(state): State<AppState>, req: Request<Body>) -> Response<Body> {
    let payment_amount = U256::from(1000);

    // Parse headers
    let updated_channel = match parse_headers_axum(req, payment_amount).await {
        Ok((signed_request, _parts)) => {
            let _body_bytes = signed_request.body_bytes.clone();
            match verify_and_update_channel(&state.channel_state, signed_request).await {
                Ok(updated_channel) => updated_channel,
                Err(e) => return Response::builder().status(e).body(Body::empty()).unwrap(),
            }
        }
        Err(e) => return Response::builder().status(e).body(Body::empty()).unwrap(),
    };

    let ai = match state.env.ai("Ai") {
        Ok(ai) => ai,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(e.to_string()))
                .unwrap()
        }
    };

    #[derive(serde::Serialize)]
    struct Input {
        prompt: String,
    }

    #[derive(serde::Deserialize)]
    struct Output {
        response: String,
    }

    let res: std::result::Result<Output, Error> = ai
        .run::<Input, Output>(
            "@cf/meta/llama-3.1-8b-instruct",
            Input {
                prompt: String::from("What is the origin of the phrase Hello, World"),
            },
        )
        .await;

    match res {
        Ok(res) => {
            let response = Response::builder()
                .status(200)
                .body(Body::from(res.response))
                .unwrap();
            modify_headers_axum(response, &updated_channel)
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}
