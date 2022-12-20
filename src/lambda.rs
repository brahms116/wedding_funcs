use serde_json::{json, Value};

pub struct HttpError {
    pub status_code: i32,
    pub msg: Option<String>,
}

impl Into<Value> for HttpError {
    fn into(self) -> Value {
        let msg = match self.msg {
            Some(description) => json!(description),
            None => Value::Null,
        };
        lambda_response(json!({ "msg": msg }), self.status_code)
    }
}

pub fn lambda_response(body: Value, code: i32) -> Value {
    return json!({
        "statusCode":code,
        "headers":{
            "Content-Type":"application/json",
            "Access-Control-Allow-Origin":"*"
        },
        "body":body.to_string()
    });
}