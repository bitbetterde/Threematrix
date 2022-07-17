use log::debug;
use std::{error::Error, future::Future};
use tokio::time::{sleep, Duration};

pub async fn retry_request<B: Future<Output = Result<T, E>>, T, E: Error>(
    callback: impl Fn() -> B,
    delay_in_ms: u64,
    retries: u32,
) -> Result<T, E> {
    let mut result = callback().await;
    let mut retry_counter = retries;

    while let Err(msg) = &result {
        if retry_counter == 0 {
            break;
        }
        debug!("Retrying due to error: {}", msg);
        sleep(Duration::from_millis(delay_in_ms)).await;
        retry_counter = retry_counter - 1;
        result = callback().await;
    }
    return result;
}
