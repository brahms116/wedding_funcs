mod lambda;
mod lambda_integration;

use std::env;

use lambda::*;
use lambda_runtime::{service_fn, LambdaEvent};
use serde_json::{json, Value};
use wedding_funcs::*;

fn parse_params_from_request(request: &Value) -> Option<Payload> {
    let body = request.get("body")?;
    serde_json::from_value(body.clone()).ok()?
}

type StdErr = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), StdErr> {
    let func = service_fn(handle);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handle(event: LambdaEvent<Value>) -> Result<Value, StdErr> {
    println!("Lambda function called again");
    let (event, _context) = event.into_parts();
    let params = parse_params_from_request(&event);

    if let None = params {
        let err: HttpError = HttpError::from(ApiErr::ArgumentErr(
            "Could not parse into a valid params or function call".to_string(),
        ));
        return Ok(err.into());
    }
    let params = params.expect("Should handle none");

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
        Ok(value) => Ok(lambda_response(json!(value), 200)),
        Err(err) => Ok(HttpError::from(err).into()),
    }
}
