use serde_json::Value;

pub struct Call {
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

impl Call {
    pub fn from_json(val: &Value) -> Result<Call, String> {
        match val {
            Value::Object(map) => {
                let method = map.get("method");
                if method.is_none() {
                    return Err("No method name given".into());
                }
                let method = match method.unwrap() {
                    Value::String(s) => s.clone(),
                    _ => return Err("method was not  a string".into()),
                };
                let params = map.get("params").map(|x| x.clone());
                let id = map.get("id").clone().map(|x| x.clone());

                Ok(Call { method, params, id })
            }
            _ => Err("Value wasn't an object".into()),
        }
    }

    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("jsonrpc".into(), Value::String("2.0".into()));
        map.insert("method".into(), Value::String(self.method.clone()));
        if let Some(params) = &self.params {
            map.insert("params".into(), params.clone());
        }
        if let Some(id) = &self.id {
            map.insert("id".into(), id.clone());
        }

        Value::Object(map)
    }
}

pub fn make_result_response(id: Option<Value>, result: Value) -> Value {
    let mut response = serde_json::Map::new();
    response.insert("jsonrpc".into(), "2.0".into());
    response.insert("result".into(), result);
    if let Some(id) = id {
        response.insert("id".into(), id);
    }

    Value::Object(response)
}

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST_ERROR: i64 = -32600;
pub const METHOD_NOT_FOUND_ERROR: i64 = -32601;
pub const INVALID_PARAMS_ERROR: i64 = -32602;
pub const SERVER_ERROR: i64 = -32000;

// not needed right now
#[allow(dead_code)]
pub const INTERNAL_ERROR: i64 = -32603;

pub struct Error {
    code: i64,
    message: String,
    data: Option<Value>,
}

pub fn make_error(code: i64, message: String, data: Option<Value>) -> Error {
    Error {
        code,
        message,
        data,
    }
}

pub fn make_error_response(id: Option<Value>, error: Error) -> Value {
    let mut json_err = serde_json::Map::new();
    json_err.insert(
        "code".into(),
        Value::Number(serde_json::Number::from(error.code)),
    );
    json_err.insert("message".into(), Value::String(error.message.clone()));

    if let Some(data) = error.data {
        json_err.insert("data".into(), data.clone());
    }

    let mut response = serde_json::Map::new();
    response.insert("jsonrpc".into(), "2.0".into());
    response.insert("error".into(), Value::Object(json_err));
    if let Some(id) = id {
        response.insert("id".into(), id);
    }

    Value::Object(response)
}

pub fn get_next_call(source: &mut dyn std::io::Read) -> serde_json::Result<Result<Call, String>> {
    match serde_json::from_reader(source) {
        Ok(v) => {
            let v: Value = v;
            Ok(Call::from_json(&v))
        }
        Err(e) => Err(e),
    }
}
