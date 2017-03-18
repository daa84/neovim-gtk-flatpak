
use super::value::{read_value, write_value, Value, Integer};
use std::io;
use std::io::{Read, Write};
use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum RpcMessage {
    RpcRequest {
        msgid: u64,
        method: String,
        params: Vec<Value>,
    }, // 0
    RpcResponse {
        msgid: u64,
        error: Value,
        result: Value,
    }, // 1
    RpcNotification { method: String, params: Vec<Value> }, // 2
}

macro_rules! try_str {
    ($exp:expr, $msg:expr) => (match $exp {
        Value::String(ref val) => val.to_owned(),
        _ => return Err(Box::new(io::Error::new(io::ErrorKind::Other, $msg)))
    })
}

macro_rules! try_int {
    ($exp:expr, $msg:expr) => (match $exp {
        Value::Integer(Integer::U64(val)) => val,
        _ => return Err(Box::new(io::Error::new(io::ErrorKind::Other, $msg)))
    })
}

macro_rules! try_arr {
    ($exp:expr, $msg:expr) => (match $exp {
        Value::Array(ref arr) => arr.to_owned(),
        _ => return Err(Box::new(io::Error::new(io::ErrorKind::Other, $msg)))
    })
}

macro_rules! rpc_args {
    ($($e:expr), *) => {{
        let mut vec = Vec::new();
        $(
            vec.push($e.into_val());
        )*
        vec.into_val()
    }}
}

pub fn decode<R: Read>(reader: &mut R) -> Result<RpcMessage, Box<Error>> {
    let arr = try_arr!(read_value(reader)?, "Rpc message must be array");
    match try_int!(arr[0], "Can't find message type") {
        0 => {
            let msgid = try_int!(arr[1], "msgid not found");
            let method = try_str!(arr[2], "method not found");
            let params = try_arr!(arr[3], "params not found");
            Ok(RpcMessage::RpcRequest {
                msgid: msgid,
                method: method,
                params: params,
            })
        }
        1 => {
            let msgid = try_int!(arr[1], "msgid not found");
            Ok(RpcMessage::RpcResponse {
                msgid: msgid,
                error: arr[2].to_owned(),
                result: arr[3].to_owned(),
            })
        }
        2 => {
            let method = try_str!(arr[1], "method not found");
            let params = try_arr!(arr[2], "params not found");
            Ok(RpcMessage::RpcNotification {
                method: method,
                params: params,
            })

        }
        _ => Err(Box::new(io::Error::new(io::ErrorKind::Other, "Not nown type"))),
    }
}

pub fn encode<W: Write>(writer: &mut W, msg: &RpcMessage) -> Result<(), Box<Error>> {
    match msg {
        &RpcMessage::RpcRequest { msgid, ref method, ref params } => {
            let val = rpc_args!(0, msgid, method.to_owned(), params.to_owned());
            write_value(writer, &val)?;
        }
        &RpcMessage::RpcResponse { msgid, ref error, ref result } => {
            let val = rpc_args!(1, msgid, error.to_owned(), result.to_owned());
            write_value(writer, &val)?;
        }
        &RpcMessage::RpcNotification { ref method, ref params } => {
            let val = rpc_args!(2, method.to_owned(), params.to_owned());
            write_value(writer, &val)?;
        }
    };
    Ok(())
}

pub trait FromVal<T> {
    fn from_val(T) -> Self;
}

impl FromVal<Value> for () {
    fn from_val(_: Value) -> Self {
        ()
    }
}

impl FromVal<Value> for Value {
    fn from_val(val: Value) -> Self {
        val
    }
}

impl FromVal<Value> for Vec<(Value, Value)> {
    fn from_val(val: Value) -> Self {
        if let Value::Map(vec) = val {
            return vec;
        }
        panic!("Not supported value for map");
    }
}

impl<T: FromVal<Value>> FromVal<Value> for Vec<T> {
    fn from_val(val: Value) -> Self {
        if let Value::Array(arr) = val {
            return arr.iter()
                .map(|v| T::from_val(v.clone()))
                .collect();
        }
        panic!("Can't convert to string");
    }
}

impl FromVal<Value> for (u64, u64) {
    fn from_val(val: Value) -> Self {
        if let Value::Array(res) = val {
            if res.len() != 2 {
                panic!("Array length must be 2");
            }
            let p1_val: &Value = &res[0];
            let p2_val: &Value = &res[1];
            let p1 = if let &Value::Integer(Integer::U64(ref p1)) = p1_val {
                *p1
            } else {
                panic!("Can't get u64 value at position 0");
            };

            let p2 = if let &Value::Integer(Integer::U64(ref p2)) = p2_val {
                *p2
            } else {
                panic!("Can't get u64 value at position 1");
            };

            return (p1, p2);
        }
        panic!("Can't convert to string");
    }
}

impl FromVal<Value> for bool {
    fn from_val(val: Value) -> Self {
        if let Value::Boolean(res) = val {
            return res;
        }
        panic!("Can't convert to string");
    }
}

impl FromVal<Value> for String {
    fn from_val(val: Value) -> Self {
        if let Value::String(res) = val {
            return res;
        }
        panic!("Can't convert to string");
    }
}

impl FromVal<Value> for u64 {
    fn from_val(val: Value) -> Self {
        if let Value::Integer(Integer::U64(res)) = val {
            return res;
        }
        panic!("Can't convert to string");
    }
}

pub trait IntoVal<T> {
    fn into_val(self) -> T;
}

impl<'a> IntoVal<Value> for &'a str {
    fn into_val(self) -> Value {
        Value::String(self.to_string())
    }
}

impl IntoVal<Value> for Vec<String> {
    fn into_val(self) -> Value {
        Value::Array(self.iter().map(|v| Value::String(v.to_string())).collect())
    }
}

impl IntoVal<Value> for Vec<Value> {
    fn into_val(self) -> Value {
        Value::Array(self)
    }
}

impl IntoVal<Value> for (u64, u64) {
    fn into_val(self) -> Value {
        Value::Array(vec![self.0.into_val(), self.1.into_val()])
    }
}

impl IntoVal<Value> for bool {
    fn into_val(self) -> Value {
        Value::Boolean(self)
    }
}

impl IntoVal<Value> for u64 {
    fn into_val(self) -> Value {
        Value::Integer(Integer::U64(self))
    }
}

impl IntoVal<Value> for String {
    fn into_val(self) -> Value {
        Value::String(self)
    }
}

impl IntoVal<Value> for Value {
    fn into_val(self) -> Value {
        self
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, SeekFrom, Seek};
    use super::*;

    #[test]
    fn request_test() {
        let msg = RpcMessage::RpcRequest {
            msgid: 1,
            method: "test_method".to_owned(),
            params: vec![],
        };

        let mut buff = Cursor::new(vec![]);
        encode(&mut buff, &msg).unwrap();

        buff.seek(SeekFrom::Start(0)).unwrap();
        let msg_dest = decode(&mut buff).unwrap();
        assert_eq!(msg, msg_dest);
    }
}
