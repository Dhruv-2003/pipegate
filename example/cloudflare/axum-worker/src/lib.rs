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
struct MiddlewareConfig {
    payment_amount: U256,
}

#[derive(Clone)]
pub(crate) struct AppState {
    channel_state: ChannelState,
    env: Env,
    middleware_config: MiddlewareConfig,
}

fn router(env: Env) -> Router {
    let state = ChannelState::new("https://base-sepolia-rpc.publicnode.com".parse().unwrap());
    let payment_amount = U256::from(1000);

    let app_state = AppState {
        channel_state: state.clone(),
        env,
        middleware_config: MiddlewareConfig { payment_amount },
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
    let payment_amount = state.middleware_config.payment_amount;
    let state = state.channel_state;
    // Parse headers
    let (parts, body) = req.into_parts();

    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed: Body decode");
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap();
        }
    };

    match parse_headers_axum(&parts.headers, body_bytes, payment_amount).await {
        Ok(signed_request) => {
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

async fn generate(State(state): State<AppState>, _req: Request<Body>) -> Response<Body> {
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
            response
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e.to_string()))
            .unwrap(),
    }
}
