mod lambda;
mod lambda_integration;

use std::env;

use lambda::*;
use lambda_runtime::{service_fn, LambdaEvent};
use serde_json::{json, Value};
use tracing::{event, Level};
use tracing_subscriber;
use wedding_funcs::*;

type StdErr = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), StdErr> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .json()
        .init();
    let func = service_fn(handle);
    lambda_runtime::run(func).await?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(body))]
async fn handle(event: LambdaEvent<Value>) -> Result<Value, StdErr> {
    let (event, _context) = event.into_parts();
    let body = event.get("body");
    tracing::Span::current().record("body", format!("{:?}", body));

    event!(Level::INFO, "Lambda function called");

    if let None = body {
        event!(Level::WARN, "Missing body");
        return Ok(HttpError::from(ApiErr::ArgumentErr("Missing body".into())).into());
    }

    let body_str = body.expect("Should hanlde None").as_str();

    if let None = body_str {
        event!(Level::WARN, "Non-string body");
        return Ok(HttpError::from(ApiErr::ArgumentErr("Body is not a string".into())).into());
    }

    let params: Result<Payload, _> = serde_json::from_str(body_str.expect("Should handle None"));

    if let Err(err) = params {
        let err: HttpError = HttpError::from(ApiErr::ArgumentErr(format!(
            "Could not parse into a function with parameters: {}",
            err
        )));
        event!(Level::WARN, "Invalid parameters");
        return Ok(err.into());
    }

    let params = params.expect("Should handle err");

    let uri = env::var("WED_POSTGRES_URI").expect("Uri should be defined in env");
    let (client, connection) = tokio_postgres::connect(&uri, tokio_postgres::NoTls)
        .await
        .expect("Connection should not fail");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let db = DB { client: &client };
    let result = handle_request(params, db).await;

    match result {
        Ok(value) => {
            event!(Level::INFO, "function result success {:?}", value);
            Ok(lambda_response(json!({ "data": value }), 200))
        }
        Err(err) => {
            event!(Level::ERROR, "function result err {}", err);
            Ok(HttpError::from(err).into())
        }
    }
}
