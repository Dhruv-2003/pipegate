use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use alloy::primitives::{Address, FixedBytes};
use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use pipegate::{
    types::{OneTimePaymentConfig, Url, U256},
    utils::headers::parse_tx_headers_axum,
    verify::verify_tx,
};
use tokio::sync::RwLock;

pub mod errors;

#[derive(Clone)]
struct AwsLocalConfig {
    client: Client,
    bucket_name: String,
    object_key: String,
}

#[derive(Clone)]
struct AppState {
    state: Arc<RwLock<HashMap<FixedBytes<32>, String>>>,
    payment_config: OneTimePaymentConfig,
    aws_local_config: AwsLocalConfig,
}

#[tokio::main]
pub async fn main() {
    let rpc_url: Url = "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    let onetime_payment_config = OneTimePaymentConfig {
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        token_address: Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(),
        amount: U256::from(1000000), // 1 USDC
        period: U256::from(0),
        rpc_url: rpc_url.to_string(),
    };

    let state = Arc::new(RwLock::new(HashMap::new()));

    let s3_client = setup_s3_client().await;
    let bucket_name = "s3-demo-pipegate";
    let object_key = "OnetimePaymentArchitecture.png";

    let aws_local_config = AwsLocalConfig {
        client: s3_client,
        bucket_name: bucket_name.to_string(),
        object_key: object_key.to_string(),
    };

    let app_state = AppState {
        state,
        payment_config: onetime_payment_config,
        aws_local_config,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/request", post(request))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("Listening on: http://localhost:3000");
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn request(State(app_state): State<AppState>, request: Request<Body>) -> impl IntoResponse {
    println!("Request received");

    let headers = request.headers().clone();

    println!("Extracting headers");
    let signed_payment_headers = match parse_tx_headers_axum(&headers).await {
        Ok(headers) => headers,
        Err(error) => {
            println!("Failed to parse headers: {:?}", error.to_string());
            return (error, format!("Error: {:?}", error.to_string())).into_response();
        }
    };

    let tx_hash = signed_payment_headers.tx_hash;

    println!("Verifying payment");
    match verify_tx(signed_payment_headers, app_state.payment_config).await {
        Ok(result) => {
            // If the headers are verified, then we can generate a presigned URL or return an already generated URL
            if result == true {
                let mut state = app_state.state.write().await;

                // If the URL is already generated, return the URL
                if let Some(url) = state.get(&tx_hash) {
                    return (StatusCode::OK, url.clone()).into_response();
                }

                let aws_config = &app_state.aws_local_config;

                println!("Generating presigned URL");
                let presigned_url = generate_presigned_url(
                    aws_config.client.clone(),
                    &aws_config.bucket_name,
                    &aws_config.object_key,
                )
                .await;

                match presigned_url {
                    Some(url) => {
                        state.insert(tx_hash, url.clone());
                        println!("Presigned URL: {}", url);
                        (StatusCode::OK, url).into_response()
                    }
                    None => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to generate URL".to_string(),
                    )
                        .into_response(),
                }
            } else {
                // If the headers are not verified, return an error
                println!("Failed to verify payment");
                (
                    StatusCode::UNAUTHORIZED,
                    "Payment verification failed".to_string(),
                )
                    .into_response()
            }
        }
        Err(error) => {
            println!("Failed to verify payment: {:?}", error.to_string());
            let error_body = error.to_string();
            (StatusCode::from(error), format!("Error: {:?}", error_body)).into_response()
        }
    }
}

//* AWS related functions */
// Setup S3 client
async fn setup_s3_client() -> Client {
    let region = RegionProviderChain::first_try(Region::new("eu-north-1"));
    let config = aws_config::from_env().region(region).load().await;
    Client::new(&config)
}

// Generate presigned URL
async fn generate_presigned_url(
    client: Client,
    bucket_name: &str,
    object_key: &str,
) -> Option<String> {
    let presigning_config = PresigningConfig::builder()
        .expires_in(Duration::from_secs(3600))
        .build()
        .unwrap();

    let presigned_url = client
        .get_object()
        .bucket(bucket_name)
        .key(object_key)
        .presigned(presigning_config)
        .await;

    match presigned_url {
        Ok(response) => Some(response.uri().to_string()),
        Err(error) => {
            eprintln!("Failed to generate presigned URL: {:?}", error);
            None
        }
    }
}

// Upload object
// async fn _upload_object(
//     client: &aws_sdk_s3::Client,
//     bucket_name: &str,
//     file_name: &str,
//     key: &str,
// ) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, aws_sdk_s3::Error> {
//     let body = aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(file_name)).await;

//     client
//         .put_object()
//         .bucket(bucket_name)
//         .key(key)
//         .body(body.unwrap())
//         .send()
//         .await;
// }

// List objects
pub async fn list_objects(client: &aws_sdk_s3::Client, bucket: &str) {
    let mut response = client
        .list_objects_v2()
        .bucket(bucket.to_owned())
        .max_keys(10) // In this example, go 10 at a time.
        .into_paginator()
        .send();

    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                for object in output.contents() {
                    println!(" - {}", object.key().unwrap_or("Unknown"));
                }
            }
            Err(err) => {
                eprintln!("{err:?}")
            }
        }
    }
}
