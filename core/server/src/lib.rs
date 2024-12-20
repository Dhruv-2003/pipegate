pub mod channel;
pub mod error;
pub mod middleware;
pub mod types;
pub mod utils;
pub mod verify;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_and_update() {}
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
