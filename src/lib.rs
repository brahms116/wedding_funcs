mod api;
mod db;
mod func;
mod models;

pub use api::*;
pub use func::*;
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    #[tokio::test]
    async fn test_connection() {
        let uri = env::var("DEV_POSTGRES_URI").expect("Uri should be defined for test");
        let (_client, connection) =
            tokio_postgres::connect(&format!("{}/wedding_dev", uri), tokio_postgres::NoTls)
                .await
                .expect("Connection should not fail");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
    }
}
