//! Photon's binary value encoding "Protocol16" (WebSocket sub-protocol `GpBinaryV16`), as the
//! original client's Photon3Unity3D library writes it. Big-endian; every value is a type code
//! followed by its payload. Spec and evidence: `docs/protocol.md`.
//!
//! Hashtables and dictionaries keep their entry order, so a decoded message encodes back to the
//! same bytes (checked against the recorded session in the tests).

use anyhow::{Result, bail};
use glam::{Quat, Vec3};

pub const NULL: u8 = 42; // '*'
pub const DICTIONARY: u8 = 68; // 'D'
pub const STRING_ARRAY: u8 = 97; // 'a'
pub const BYTE: u8 = 98; // 'b'
pub const CUSTOM: u8 = 99; // 'c'
pub const DOUBLE: u8 = 100; // 'd'
pub const FLOAT: u8 = 102; // 'f'
pub const HASHTABLE: u8 = 104; // 'h'
pub const INT: u8 = 105; // 'i'
pub const SHORT: u8 = 107; // 'k'
pub const LONG: u8 = 108; // 'l'
pub const INT_ARRAY: u8 = 110; // 'n'
pub const BOOL: u8 = 111; // 'o'
pub const STRING: u8 = 115; // 's'
pub const BYTE_ARRAY: u8 = 120; // 'x'
pub const ARRAY: u8 = 121; // 'y'
pub const OBJECT_ARRAY: u8 = 122; // 'z'

/// PUN's custom types (`CustomTypes.Register`): Unity structs and `PhotonPlayer`.
pub const CUSTOM_VECTOR3: u8 = b'V';
pub const CUSTOM_QUATERNION: u8 = b'Q';
pub const CUSTOM_PLAYER: u8 = b'P';

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Byte(u8),
    Bool(bool),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    StringArray(Vec<String>),
    /// `object[]`: every element with its own type code.
    ObjectArray(Vec<Value>),
    /// A typed array (`'y'`): the element type once, then the payloads. Custom elements are
    /// `Value::Custom` sharing one custom code.
    Array(u8, Vec<Value>),
    Hashtable(Vec<(Value, Value)>),
    /// `Dictionary<K, V>`: key and value type codes (0 or `'*'` = each entry typed).
    Dictionary(u8, u8, Vec<(Value, Value)>),
    Custom(u8, Vec<u8>),
}

impl Value {
    pub fn str(text: &str) -> Value {
        Value::String(text.to_string())
    }

    /// `Vector3` as PUN serializes it (x, y, z).
    pub fn vec3(v: Vec3) -> Value {
        let mut data = Vec::with_capacity(12);
        for f in [v.x, v.y, v.z] {
            data.extend(f.to_be_bytes());
        }
        Value::Custom(CUSTOM_VECTOR3, data)
    }

    /// `Quaternion` as PUN serializes it (w, x, y, z).
    pub fn quat(q: Quat) -> Value {
        let mut data = Vec::with_capacity(16);
        for f in [q.w, q.x, q.y, q.z] {
            data.extend(f.to_be_bytes());
        }
        Value::Custom(CUSTOM_QUATERNION, data)
    }

    /// `PhotonPlayer` as PUN serializes it (its actor number).
    pub fn player(actor: i32) -> Value {
        Value::Custom(CUSTOM_PLAYER, actor.to_be_bytes().to_vec())
    }

    fn floats<const N: usize>(&self, code: u8) -> Option<[f32; N]> {
        let Value::Custom(c, data) = self else {
            return None;
        };
        if *c != code || data.len() != N * 4 {
            return None;
        }
        let mut out = [0.0; N];
        for (i, chunk) in data.as_chunks::<4>().0.iter().enumerate() {
            out[i] = f32::from_be_bytes(*chunk);
        }
        Some(out)
    }

    pub fn as_vec3(&self) -> Option<Vec3> {
        self.floats::<3>(CUSTOM_VECTOR3).map(Vec3::from_array)
    }

    pub fn as_quat(&self) -> Option<Quat> {
        self.floats::<4>(CUSTOM_QUATERNION)
            .map(|[w, x, y, z]| Quat::from_xyzw(x, y, z, w))
    }

    pub fn as_player(&self) -> Option<i32> {
        match self {
            Value::Custom(CUSTOM_PLAYER, data) if data.len() == 4 => {
                Some(i32::from_be_bytes([data[0], data[1], data[2], data[3]]))
            }
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Any integer type, as C#'s implicit widening allows.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Byte(v) => Some(i32::from(*v)),
            Value::Short(v) => Some(i32::from(*v)),
            Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Double(v) => Some(*v as f32),
            _ => self.as_i32().map(|v| v as f32),
        }
    }

    /// Elements of an `object[]` or a typed array.
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::ObjectArray(items) | Value::Array(_, items) => Some(items),
            _ => None,
        }
    }

    /// Entries of a hashtable or dictionary.
    pub fn entries(&self) -> Option<&[(Value, Value)]> {
        match self {
            Value::Hashtable(entries) | Value::Dictionary(_, _, entries) => Some(entries),
            _ => None,
        }
    }

    /// The entry whose key is the byte `key` (PUN's content tables use byte keys).
    pub fn get_byte(&self, key: u8) -> Option<&Value> {
        self.entries()?
            .iter()
            .find(|(k, _)| *k == Value::Byte(key))
            .map(|(_, v)| v)
    }

    /// The entry whose key is the string `key` (custom properties).
    pub fn get_str(&self, key: &str) -> Option<&Value> {
        self.entries()?
            .iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .map(|(_, v)| v)
    }

    fn type_code(&self) -> u8 {
        match self {
            Value::Null => NULL,
            Value::Byte(_) => BYTE,
            Value::Bool(_) => BOOL,
            Value::Short(_) => SHORT,
            Value::Int(_) => INT,
            Value::Long(_) => LONG,
            Value::Float(_) => FLOAT,
            Value::Double(_) => DOUBLE,
            Value::String(_) => STRING,
            Value::ByteArray(_) => BYTE_ARRAY,
            Value::IntArray(_) => INT_ARRAY,
            Value::StringArray(_) => STRING_ARRAY,
            Value::ObjectArray(_) => OBJECT_ARRAY,
            Value::Array(..) => ARRAY,
            Value::Hashtable(_) => HASHTABLE,
            Value::Dictionary(..) => DICTIONARY,
            Value::Custom(..) => CUSTOM,
        }
    }
}

pub struct Reader<'a> {
    data: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Reader<'a> {
        Reader { data, pos: 0 }
    }

    pub fn at_end(&self) -> bool {
        self.pos == self.data.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.data.len() {
            bail!(
                "Protocol16: read of {n} bytes past the end at {} of {}",
                self.pos,
                self.data.len()
            );
        }
        let chunk = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(chunk)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut out = [0; N];
        out.copy_from_slice(self.take(N)?);
        Ok(out)
    }

    pub fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn short(&mut self) -> Result<i16> {
        Ok(i16::from_be_bytes(self.array()?))
    }

    fn int(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.array()?))
    }

    fn count(&mut self) -> Result<usize> {
        let n = self.short()?;
        if n < 0 {
            bail!("Protocol16: negative length {n}");
        }
        Ok(n as usize)
    }

    fn string(&mut self) -> Result<String> {
        let n = u16::from_be_bytes(self.array()?) as usize;
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }

    pub fn typed(&mut self) -> Result<Value> {
        let t = self.byte()?;
        self.value(t)
    }

    /// An operation's or event's parameter table: count, then (byte key, typed value).
    pub fn params(&mut self) -> Result<Vec<(u8, Value)>> {
        let n = self.count()?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let key = self.byte()?;
            out.push((key, self.typed()?));
        }
        Ok(out)
    }

    fn element(&mut self, t: u8) -> Result<Value> {
        if t == 0 || t == NULL {
            self.typed()
        } else {
            self.value(t)
        }
    }

    pub fn value(&mut self, t: u8) -> Result<Value> {
        Ok(match t {
            0 | NULL => Value::Null,
            BYTE => Value::Byte(self.byte()?),
            BOOL => Value::Bool(self.byte()? != 0),
            SHORT => Value::Short(self.short()?),
            INT => Value::Int(self.int()?),
            LONG => Value::Long(i64::from_be_bytes(self.array()?)),
            FLOAT => Value::Float(f32::from_be_bytes(self.array()?)),
            DOUBLE => Value::Double(f64::from_be_bytes(self.array()?)),
            STRING => Value::String(self.string()?),
            BYTE_ARRAY => {
                let n = self.int()?;
                if n < 0 {
                    bail!("Protocol16: negative byte[] length {n}");
                }
                Value::ByteArray(self.take(n as usize)?.to_vec())
            }
            INT_ARRAY => {
                let n = self.int()?;
                if n < 0 {
                    bail!("Protocol16: negative int[] length {n}");
                }
                Value::IntArray((0..n).map(|_| self.int()).collect::<Result<_>>()?)
            }
            STRING_ARRAY => {
                let n = self.count()?;
                Value::StringArray((0..n).map(|_| self.string()).collect::<Result<_>>()?)
            }
            OBJECT_ARRAY => {
                let n = self.count()?;
                Value::ObjectArray((0..n).map(|_| self.typed()).collect::<Result<_>>()?)
            }
            ARRAY => {
                let n = self.count()?;
                let element = self.byte()?;
                if element == CUSTOM {
                    let code = self.byte()?;
                    let mut items = Vec::with_capacity(n);
                    for _ in 0..n {
                        let len = self.count()?;
                        items.push(Value::Custom(code, self.take(len)?.to_vec()));
                    }
                    Value::Array(element, items)
                } else if matches!(element, ARRAY | DICTIONARY | 0 | NULL) {
                    bail!("Protocol16: nested typed arrays of type {element} are not supported");
                } else {
                    Value::Array(
                        element,
                        (0..n).map(|_| self.value(element)).collect::<Result<_>>()?,
                    )
                }
            }
            HASHTABLE => {
                let n = self.count()?;
                let mut entries = Vec::with_capacity(n);
                for _ in 0..n {
                    let key = self.typed()?;
                    entries.push((key, self.typed()?));
                }
                Value::Hashtable(entries)
            }
            DICTIONARY => {
                let (key_type, value_type) = (self.byte()?, self.byte()?);
                let n = self.count()?;
                let mut entries = Vec::with_capacity(n);
                for _ in 0..n {
                    let key = self.element(key_type)?;
                    entries.push((key, self.element(value_type)?));
                }
                Value::Dictionary(key_type, value_type, entries)
            }
            CUSTOM => {
                let code = self.byte()?;
                let len = self.count()?;
                Value::Custom(code, self.take(len)?.to_vec())
            }
            other => bail!(
                "Protocol16: unknown type code {other} at {}",
                self.pos.saturating_sub(1)
            ),
        })
    }
}

fn write_len(out: &mut Vec<u8>, n: usize) {
    out.extend((n.min(i16::MAX as usize) as i16).to_be_bytes());
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend((bytes.len().min(u16::MAX as usize) as u16).to_be_bytes());
    out.extend(bytes);
}

pub fn write_typed(out: &mut Vec<u8>, value: &Value) {
    out.push(value.type_code());
    write_value(out, value);
}

fn write_element(out: &mut Vec<u8>, t: u8, value: &Value) {
    if t == 0 || t == NULL {
        write_typed(out, value);
    } else {
        write_value(out, value);
    }
}

/// A value's payload (without its type code).
pub fn write_value(out: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Null => {}
        Value::Byte(v) => out.push(*v),
        Value::Bool(v) => out.push(u8::from(*v)),
        Value::Short(v) => out.extend(v.to_be_bytes()),
        Value::Int(v) => out.extend(v.to_be_bytes()),
        Value::Long(v) => out.extend(v.to_be_bytes()),
        Value::Float(v) => out.extend(v.to_be_bytes()),
        Value::Double(v) => out.extend(v.to_be_bytes()),
        Value::String(s) => write_string(out, s),
        Value::ByteArray(bytes) => {
            out.extend((bytes.len() as i32).to_be_bytes());
            out.extend(bytes);
        }
        Value::IntArray(ints) => {
            out.extend((ints.len() as i32).to_be_bytes());
            for v in ints {
                out.extend(v.to_be_bytes());
            }
        }
        Value::StringArray(strings) => {
            write_len(out, strings.len());
            for s in strings {
                write_string(out, s);
            }
        }
        Value::ObjectArray(items) => {
            write_len(out, items.len());
            for item in items {
                write_typed(out, item);
            }
        }
        Value::Array(element, items) => {
            write_len(out, items.len());
            out.push(*element);
            if *element == CUSTOM {
                let code = match items.first() {
                    Some(Value::Custom(code, _)) => *code,
                    _ => 0,
                };
                out.push(code);
                for item in items {
                    if let Value::Custom(_, data) = item {
                        write_len(out, data.len());
                        out.extend(data);
                    }
                }
            } else {
                for item in items {
                    write_value(out, item);
                }
            }
        }
        Value::Hashtable(entries) => {
            write_len(out, entries.len());
            for (key, value) in entries {
                write_typed(out, key);
                write_typed(out, value);
            }
        }
        Value::Dictionary(key_type, value_type, entries) => {
            out.push(*key_type);
            out.push(*value_type);
            write_len(out, entries.len());
            for (key, value) in entries {
                write_element(out, *key_type, key);
                write_element(out, *value_type, value);
            }
        }
        Value::Custom(code, data) => {
            out.push(*code);
            write_len(out, data.len());
            out.extend(data);
        }
    }
}

pub fn write_params(out: &mut Vec<u8>, params: &[(u8, Value)]) {
    write_len(out, params.len());
    for (key, value) in params {
        out.push(*key);
        write_typed(out, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(value: Value) {
        let mut bytes = Vec::new();
        write_typed(&mut bytes, &value);
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.typed().unwrap(), value);
        assert!(reader.at_end());
    }

    #[test]
    fn values_round_trip() {
        round_trip(Value::Null);
        round_trip(Value::Byte(200));
        round_trip(Value::Bool(true));
        round_trip(Value::Short(-2));
        round_trip(Value::Int(-607461470));
        round_trip(Value::Long(1 << 40));
        round_trip(Value::Float(3.5));
        round_trip(Value::Double(3687461.5));
        round_trip(Value::str("Room 3197: TDM"));
        round_trip(Value::ByteArray(vec![1, 2, 3]));
        round_trip(Value::IntArray(vec![1, 2]));
        round_trip(Value::StringArray(vec![
            "MapName".into(),
            "GameMode".into(),
        ]));
        round_trip(Value::ObjectArray(vec![
            Value::str("syncKnife"),
            Value::Float(0.0),
        ]));
        round_trip(Value::Array(INT, vec![Value::Int(1), Value::Int(2)]));
        round_trip(Value::Hashtable(vec![
            (Value::Byte(0), Value::Int(1001)),
            (Value::str("TeamName"), Value::str("Mercenary")),
        ]));
        round_trip(Value::Dictionary(
            STRING,
            0,
            vec![(Value::str("a"), Value::Int(1))],
        ));
        round_trip(Value::vec3(Vec3::new(-28.6304, -8.04, -61.6791)));
    }

    #[test]
    fn custom_types() {
        let q = Quat::from_xyzw(0.0, -0.140901, 0.0, 0.990024);
        assert_eq!(Value::quat(q).as_quat(), Some(q));
        assert_eq!(Value::player(2).as_player(), Some(2));
        // The quaternion's bytes as recorded: w first.
        let Value::Custom(_, data) = Value::quat(Quat::IDENTITY) else {
            unreachable!()
        };
        assert_eq!(&data[..4], &1.0f32.to_be_bytes());
    }
}
