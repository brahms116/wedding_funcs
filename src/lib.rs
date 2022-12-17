mod db;
mod func;
mod models;

use func::*;
use models::*;
use std::env;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

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
