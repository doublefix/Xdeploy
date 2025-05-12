// 协议转换模块
use prost_types::{Struct, Value};
// Struct → serde_json::Value
pub fn struct_to_value(s: &Struct) -> serde_json::Value {
    let map = s
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect::<serde_json::Map<_, _>>();
    serde_json::Value::Object(map)
}

// serde_json::Value → Struct
pub fn value_to_struct(v: &serde_json::Value) -> Struct {
    if let serde_json::Value::Object(map) = v {
        let fields = map
            .iter()
            .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
            .collect();
        Struct { fields }
    } else {
        Struct {
            fields: Default::default(),
        }
    }
}

// prost_types::Value → serde_json::Value
pub fn prost_value_to_json(v: &Value) -> serde_json::Value {
    match &v.kind {
        Some(prost_types::value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(prost_types::value::Kind::NumberValue(n)) => serde_json::Value::from(*n),
        Some(prost_types::value::Kind::StringValue(s)) => serde_json::Value::from(s.clone()),
        Some(prost_types::value::Kind::BoolValue(b)) => serde_json::Value::from(*b),
        Some(prost_types::value::Kind::StructValue(s)) => struct_to_value(s),
        Some(prost_types::value::Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.iter().map(prost_value_to_json).collect())
        }
        None => serde_json::Value::Null,
    }
}

// serde_json::Value → prost_types::Value
pub fn json_to_prost_value(v: &serde_json::Value) -> Value {
    Value {
        kind: Some(match v {
            serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
            serde_json::Value::Bool(b) => prost_types::value::Kind::BoolValue(*b),
            serde_json::Value::Number(n) => {
                prost_types::value::Kind::NumberValue(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => prost_types::value::Kind::StringValue(s.clone()),
            serde_json::Value::Array(arr) => {
                prost_types::value::Kind::ListValue(prost_types::ListValue {
                    values: arr.iter().map(json_to_prost_value).collect(),
                })
            }
            serde_json::Value::Object(map) => prost_types::value::Kind::StructValue(Struct {
                fields: map
                    .iter()
                    .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                    .collect(),
            }),
        }),
    }
}
