// this code is partly copied from rmp project and modifed
//
use std;
use std::io::{Read, Write};
use rmp::Marker;
use rmp::decode::*;
use rmp::encode::*;
use std::str::{Utf8Error, from_utf8};
use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Integer {
    /// Every non-negative integer is treated as u64, even if it fits in i64.
    U64(u64),
    /// Every negative integer is treated as i64.
    I64(i64),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Float {
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Nil represents nil.
    Nil,
    /// Boolean represents true or false.
    Boolean(bool),
    /// Integer represents an integer.
    Integer(Integer),
    /// Float represents a floating point number.
    Float(Float),
    /// String extending Raw type represents a UTF-8 string.
    String(String),
    /// Binary extending Raw type represents a byte array.
    Binary(Vec<u8>),
    /// Array represents a sequence of objects.
    Array(Vec<Value>),
    /// Map represents key-value pairs of objects.
    Map(Vec<(Value, Value)>),
    /// Extended implements Extension interface: represents a tuple of type information and a byte
    /// array where type information is an integer whose meaning is defined by applications.
    Ext(i8, Vec<u8>),
}

#[derive(Debug)]
pub enum ReadValueError {
    InvalidRead(ValueReadError),
    InvalidString(Utf8Error, String),
}

impl error::Error for ReadValueError {
    fn description(&self) -> &str {
        match *self {
            ReadValueError::InvalidRead(..) => "failed to read MessagePack data",
            ReadValueError::InvalidString(..) => "failed to parse utf8 string",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ReadValueError::InvalidRead(ref err) => Some(err),
            ReadValueError::InvalidString(..) => None,
        }
    }
}

impl Display for ReadValueError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            &ReadValueError::InvalidRead(ref e) => error::Error::description(e).fmt(f),
            &ReadValueError::InvalidString(ref e, ref s) => {
                write!(f,
                       "invalid utf-8: invalid byte near index {}, for string: '{}'",
                       e.valid_up_to(),
                       s)
            }
        }
    }
}

impl From<ValueReadError> for ReadValueError {
    fn from(e: ValueReadError) -> Self {
        ReadValueError::InvalidRead(e)
    }
}

#[derive(Debug)]
pub enum WriteValueError {
    InvalidWrite(ValueWriteError),
}

impl error::Error for WriteValueError {
    fn description(&self) -> &str {
        match *self {
            WriteValueError::InvalidWrite(..) => "failed to write MessagePack data",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            WriteValueError::InvalidWrite(ref err) => Some(err),
        }
    }
}

impl Display for WriteValueError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            &WriteValueError::InvalidWrite(ref e) => error::Error::description(e).fmt(f),
        }
    }
}

impl From<ValueWriteError> for WriteValueError {
    fn from(e: ValueWriteError) -> Self {
        WriteValueError::InvalidWrite(e)
    }
}

impl From<std::io::Error> for WriteValueError {
    fn from(e: std::io::Error) -> Self {
        WriteValueError::InvalidWrite(ValueWriteError::InvalidDataWrite(e))
    }
}

/// Implements human-readable value formatting.
impl ::std::fmt::Display for Value {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Integer(Integer::U64(val)) => write!(f, "{}", val),
            Value::Integer(Integer::I64(val)) => write!(f, "{}", val),
            Value::Float(Float::F32(val)) => write!(f, "{}", val),
            Value::Float(Float::F64(val)) => write!(f, "{}", val),
            Value::String(ref val) => write!(f, "\"{}\"", val),
            Value::Binary(ref val) => write!(f, "{:?}", val),
            Value::Array(ref vec) => {
                let res = vec.iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "[{}]", res)
            }
            Value::Map(ref vec) => {
                write!(f, "{{")?;

                match vec.iter().take(1).next() {
                    Some(&(ref k, ref v)) => {
                        write!(f, "{}: {}", k, v)?;
                    }
                    None => {
                        write!(f, "")?;
                    }
                }

                for &(ref k, ref v) in vec.iter().skip(1) {
                    write!(f, ", {}: {}", k, v)?;
                }

                write!(f, "}}")
            }
            Value::Ext(ty, ref data) => write!(f, "[{}, {:?}]", ty, data),
        }
    }
}

fn read_str_data<'r, R>(rd: &mut R,
                        len: usize,
                        buf: &'r mut [u8])
                        -> Result<&'r str, ReadValueError>
    where R: Read
{
    debug_assert_eq!(len, buf.len());

    // Trying to copy exact `len` bytes.
    match rd.read_exact(buf) {
        Ok(()) => {
            match from_utf8(buf) {
                Ok(decoded) => Ok(decoded),
                Err(err) => {
                    Err(ReadValueError::InvalidString(err,
                                                      (*String::from_utf8_lossy(buf)).to_owned()))
                }
            }
        }
        Err(err) => Err(ReadValueError::InvalidRead(ValueReadError::InvalidDataRead(err))),
    }
}

fn read_array<R>(rd: &mut R, len: usize) -> Result<Vec<Value>, ReadValueError>
    where R: Read
{
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
        let val = read_value(rd)?;
        vec.push(val);
    }
    Ok(vec)
}

fn read_map<R>(rd: &mut R, len: usize) -> Result<Vec<(Value, Value)>, ReadValueError>
    where R: Read
{
    let mut map = Vec::with_capacity(len);

    for _ in 0..len {
        let key = read_value(rd)?;
        let value = read_value(rd)?;

        map.push((key, value));
    }

    Ok(map)
}

fn read_bin_data<R>(rd: &mut R, len: usize) -> Result<Vec<u8>, ValueReadError>
    where R: Read
{
    let mut vec = Vec::with_capacity(len);
    rd.read_exact(&mut vec).map_err(|e| ValueReadError::InvalidDataRead(e))?;
    Ok(vec)
}

fn read_ext_body<R>(rd: &mut R, len: usize) -> Result<(i8, Vec<u8>), ValueReadError>
    where R: Read
{
    let ty = read_data_i8(rd)?;
    let vec = read_bin_data(rd, len)?;
    Ok((ty, vec))
}

/// Attempts to read bytes from the given reader and interpret them as a `Value`.
///
pub fn read_value<R>(rd: &mut R) -> Result<Value, ReadValueError>
    where R: Read
{
    let val = match read_marker(rd).map_err(|e| ValueReadError::InvalidMarkerRead(e.0))? {
        Marker::Null => Value::Nil,
        Marker::True => Value::Boolean(true),
        Marker::False => Value::Boolean(false),
        Marker::FixPos(val) => Value::Integer(Integer::U64(val as u64)),
        Marker::FixNeg(val) => Value::Integer(Integer::I64(val as i64)),
        Marker::U8 => Value::Integer(Integer::U64(read_data_u8(rd)? as u64)),
        Marker::U16 => Value::Integer(Integer::U64(read_data_u16(rd)? as u64)),
        Marker::U32 => Value::Integer(Integer::U64(read_data_u32(rd)? as u64)),
        Marker::U64 => Value::Integer(Integer::U64(read_data_u64(rd)?)),
        Marker::I8 => Value::Integer(Integer::I64(read_data_i8(rd)? as i64)),
        Marker::I16 => Value::Integer(Integer::I64(read_data_i16(rd)? as i64)),
        Marker::I32 => Value::Integer(Integer::I64(read_data_i32(rd)? as i64)),
        Marker::I64 => Value::Integer(Integer::I64(read_data_i64(rd)?)),
        Marker::F32 => Value::Float(Float::F32(read_data_f32(rd)?)),
        Marker::F64 => Value::Float(Float::F64(read_data_f64(rd)?)),
        Marker::FixStr(len) => {
            let len = len as usize;
            let mut res = vec![0; len];
            let res = read_str_data(rd, len, &mut res)?.to_owned();
            Value::String(res)
        }
        Marker::Str8 => {
            let len = read_data_u8(rd)? as usize;
            let mut res = vec![0; len];
            let res = read_str_data(rd, len, &mut res)?.to_owned();
            Value::String(res)
        }
        Marker::Str16 => {
            let len = read_data_u16(rd)? as usize;
            let mut res = vec![0; len];
            let res = read_str_data(rd, len, &mut res)?.to_owned();
            Value::String(res)
        }
        Marker::Str32 => {
            let len = read_data_u32(rd)? as usize;
            let mut res = vec![0; len];
            let res = read_str_data(rd, len, &mut res)?.to_owned();
            Value::String(res)
        }
        Marker::FixArray(len) => {
            let len = len as usize;
            let vec = read_array(rd, len)?;
            Value::Array(vec)
        }
        Marker::Array16 => {
            let len = read_data_u16(rd)? as usize;
            let vec = read_array(rd, len)?;
            Value::Array(vec)
        }
        Marker::Array32 => {
            let len = read_data_u32(rd)? as usize;
            let vec = read_array(rd, len)?;
            Value::Array(vec)
        }
        Marker::FixMap(len) => {
            let len = len as usize;
            let map = read_map(rd, len)?;
            Value::Map(map)
        }
        Marker::Map16 => {
            let len = read_data_u16(rd)? as usize;
            let map = read_map(rd, len)?;
            Value::Map(map)
        }
        Marker::Map32 => {
            let len = read_data_u32(rd)? as usize;
            let map = read_map(rd, len)?;
            Value::Map(map)
        }
        Marker::Bin8 => {
            let len = read_data_u8(rd)? as usize;
            let vec = read_bin_data(rd, len)?;
            Value::Binary(vec)
        }
        Marker::Bin16 => {
            let len = read_data_u16(rd)? as usize;
            let vec = read_bin_data(rd, len)?;
            Value::Binary(vec)
        }
        Marker::Bin32 => {
            let len = read_data_u32(rd)? as usize;
            let vec = read_bin_data(rd, len)?;
            Value::Binary(vec)
        }
        Marker::FixExt1 => {
            let len = 1 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt2 => {
            let len = 2 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt4 => {
            let len = 4 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt8 => {
            let len = 8 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt16 => {
            let len = 16 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Ext8 => {
            let len = read_data_u8(rd)? as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Ext16 => {
            let len = read_data_u16(rd)? as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Ext32 => {
            let len = read_data_u32(rd)? as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Reserved => {
            return Err(ReadValueError::InvalidRead(ValueReadError::TypeMismatch(Marker::Reserved)))
        }
    };

    Ok(val)
}

/// Encodes and attempts to write the most efficient representation of the given Value.
///
pub fn write_value<W>(wr: &mut W, val: &Value) -> Result<(), WriteValueError>
    where W: Write
{
    match val {
        &Value::Nil => write_nil(wr)?,
        &Value::Boolean(val) => write_bool(wr, val)?,
        &Value::Integer(Integer::U64(val)) => {
            write_uint(wr, val)?;
        }
        &Value::Integer(Integer::I64(val)) => {
            write_sint(wr, val)?;
        }
        &Value::Float(Float::F32(val)) => write_f32(wr, val)?,
        &Value::Float(Float::F64(val)) => write_f64(wr, val)?,
        &Value::String(ref val) => {
            write_str(wr, &val)?;
        }
        &Value::Binary(ref val) => {
            write_bin(wr, &val)?;
        }
        &Value::Array(ref val) => {
            write_array_len(wr, val.len() as u32)?;
            for item in val {
                write_value(wr, item)?;
            }
        }
        &Value::Map(ref val) => {
            write_map_len(wr, val.len() as u32)?;
            for &(ref key, ref val) in val {
                write_value(wr, key)?;
                write_value(wr, val)?;
            }
        }
        &Value::Ext(ty, ref data) => {
            write_ext_meta(wr, data.len() as u32, ty)?;
            wr.write_all(data)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_string() {
        let mut buf = Vec::new();
        write_value(&mut buf, &Value::String("test1".to_owned())).unwrap();
        let mut reader = BufReader::new(buf.as_slice());
        let val = read_value(&mut reader).unwrap();
        assert_eq!(Value::String("test1".to_owned()), val);
    }
}
