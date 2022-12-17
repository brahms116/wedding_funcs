mod lambda;
mod lambda_integration;

use lambda::*;
use lambda_integration::*;
use serde_json::Value;
use wedding_funcs::*;

fn parse_params_from_request(request: &Value) -> Option<Payload> {
    let body = request.get("body")?;
    serde_json::from_value(body.clone()).ok()?
}

fn main() {}
