mod api;
mod db;
mod func;
mod models;

pub use api::*;
pub use db::*;
pub use func::*;
pub use models::*;

#[cfg(test)]
mod tests {
    use std::env;
    use openssl::ssl::{SslConnector, SslMethod};
    use postgres_openssl::MakeTlsConnector;
    #[tokio::test]
    async fn test_connection() {
        let cert_path = env::var("SSL_CERT_PATH").expect("ssl_cert_path should be defined for test");
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_ca_file(cert_path).unwrap();
        let connector = MakeTlsConnector::new(builder.build());

        let uri = env::var("WED_POSTGRES_URI").expect("Uri should be defined for test");
        let (_client, connection) =
            tokio_postgres::connect(&format!("{}", uri), connector)
                .await
                .expect("Connection should not fail");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
    }
}
