use alloy::primitives::{Bytes, U256};

use http::StatusCode;
use pipegate::{channel::ChannelState, utils::parse_headers, verify::verify_and_update_channel};
use std::time::{SystemTime, UNIX_EPOCH};
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<HttpResponse> {
    console_error_panic_hook::set_once();
    // console_log::init().expect("error initializing logger");

    // lodge an incoming request
    println!("Incoming request");

    let state = ChannelState::new("https://base-sepolia-rpc.publicnode.com".parse().unwrap());
    let payment_amount = U256::from(1000);

    // NOTE: This is a dummy value for the body_bytes, it should be ideally be derived from the HTTP request body
    let body_bytes = Bytes::from("0x");

    let request: HttpRequest = req.try_into()?;
    let (parts, _body) = request.into_parts();

    let updated_channel =
        match parse_headers(&parts.headers, body_bytes.to_vec(), payment_amount).await {
            Ok(signed_request) => {
                let _body_bytes = signed_request.body_bytes.clone();
                match verify_and_update_channel(&state, signed_request).await {
                    Ok(updated_channel) => updated_channel,
                    Err(e) => {
                        return Ok(http::Response::builder()
                            .status(e)
                            .body(Body::empty())
                            .unwrap())
                    }
                }
            }
            Err(e) => {
                return Ok(http::Response::builder()
                    .status(e)
                    .body(Body::empty())
                    .unwrap())
            }
        };

    let ai = match env.ai("Ai") {
        Ok(ai) => ai,
        Err(_) => {
            return Ok(http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap())
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
            let body_stream = Body::from_stream(futures_util::stream::once(async move {
                Ok::<Vec<u8>, Error>(res.response.to_string().into_bytes())
            }))
            .expect("Failed to create body stream");

            let mut response = http::Response::builder()
                .status(200)
                .body(body_stream)
                .unwrap();
            // modify_headers_axum(response, &updated_channel)
            let headers_mut = response.headers_mut();

            // convert the payment channel json into string and then return that in the header
            headers_mut.insert(
                "X-Payment",
                serde_json::to_string(&updated_channel)
                    .unwrap()
                    .parse()
                    .unwrap(),
            );
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            headers_mut.insert("X-Timestamp", now.to_string().parse().unwrap());

            return Ok(response);
        }
        Err(e) => {
            let body_stream = Body::from_stream(futures_util::stream::once(async move {
                Ok::<Vec<u8>, Error>(e.to_string().into_bytes())
            }))
            .expect("Failed to create body stream");

            return Ok(http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(body_stream)
                .unwrap());
        }
    }
}
