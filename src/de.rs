use std::str;

use byteorder::{BigEndian, ByteOrder};
use serde::de;
use serde::de::{Deserialize, DeserializeSeed, MapAccess, Visitor};

use error::{Error, Result};

struct Deserializer<'de> {
    index: usize,
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(bytes: &'de [u8]) -> Self {
        Self {
            index: 0,
            input: bytes,
        }
    }
}

pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    let t = T::deserialize(&mut deserializer).unwrap();
    if deserializer.done() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_length(&self) -> Result<u16> {
        let mut bytes: [u8; 2] = [0, 0];
        bytes[0] = self.input[self.index];
        bytes[1] = self.input[self.index + 1];

        let length = BigEndian::read_u16(&bytes);
        Ok(length)
    }
    fn read_length(&mut self) -> Result<u16> {
        let length = self.peek_length();
        self.index += 2;
        length
    }
    fn read_str(&mut self, count: u16) -> Result<&'de str> {
        let new_value = self.index + count as usize;
        match str::from_utf8(&self.input[self.index..new_value]) {
            Ok(string) => {
                self.index = new_value;
                Ok(&string)
            }
            Err(_) => Err(Error::BadData),
        }
    }
    fn read_next_value(&mut self) -> Result<String> {
        let length = self.read_length().unwrap();
        let value = self.read_str(length).unwrap();
        Ok(String::from(value))
    }
    fn read_next_value_as_str(&mut self) -> Result<&'de str> {
        let length = self.read_length().unwrap();
        let value = self.read_str(length).unwrap();
        Ok(value)
    }
    fn done(&self) -> bool {
        let length = self.peek_length().unwrap();
        length == 0
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(1 as i8)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        match value.as_ref() {
            "True" => visitor.visit_bool(true),
            "False" => visitor.visit_bool(false),
            _ => Err(Error::BadData),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_i8(value.parse::<i8>().unwrap())
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_i16(value.parse::<i16>().unwrap())
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_i32(value.parse::<i32>().unwrap())
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_i64(value.parse::<i64>().unwrap())
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_u8(value.parse::<u8>().unwrap())
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_u16(value.parse::<u16>().unwrap())
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_u32(value.parse::<u32>().unwrap())
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_u64(value.parse::<u64>().unwrap())
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_f32(value.parse::<f32>().unwrap())
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_f64(value.parse::<f64>().unwrap())
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_char(value.parse::<char>().unwrap())
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value_as_str().unwrap();
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.read_next_value().unwrap();
        visitor.visit_string(value)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
        //let value = visitor.visit_map(AmpAccess::new(&mut self)).unwrap();
        //Ok(value)
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        //self.deserialize_map(visitor)
        let value = visitor.visit_map(AmpAccess::new(&mut self)).unwrap();
        Ok(value)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct AmpAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> AmpAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        AmpAccess { de: de }
    }
}

impl<'a, 'de> MapAccess<'de> for AmpAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.de.done() {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

#[test]
fn test_deserialize_true() {
    let value = [
        0 as u8, 4 as u8, 'T' as u8, 'r' as u8, 'u' as u8, 'e' as u8, 0 as u8, 0 as u8,
    ];
    let expected = true;
    let actual: bool = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_false() {
    let value = [
        0 as u8, 5 as u8, 'F' as u8, 'a' as u8, 'l' as u8, 's' as u8, 'e' as u8, 0 as u8, 0 as u8,
    ];
    let expected = false;
    let actual: bool = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_i8() {
    let value = [
        0 as u8, 3 as u8, '-' as u8, '1' as u8, '5' as u8, 0 as u8, 0 as u8,
    ];
    let expected = -15;
    let actual: i8 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_i16() {
    let value = [
        0 as u8, 5 as u8, '-' as u8, '7' as u8, '1' as u8, '9' as u8, '4' as u8, 0 as u8, 0 as u8,
    ];
    let expected = -7194;
    let actual: i16 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_i32() {
    let value = [
        0 as u8, 6 as u8, '-' as u8, '7' as u8, '1' as u8, '9' as u8, '4' as u8, '9' as u8,
        0 as u8, 0 as u8,
    ];
    let expected = -71949;
    let actual: i32 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_i64() {
    let value = [
        0 as u8, 7 as u8, '-' as u8, '9' as u8, '6' as u8, '5' as u8, '5' as u8, '3' as u8,
        '7' as u8, 0 as u8, 0 as u8,
    ];
    let expected = -965537;
    let actual: i64 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_u8() {
    let value = [
        0 as u8, 3 as u8, '2' as u8, '5' as u8, '5' as u8, 0 as u8, 0 as u8,
    ];
    let expected = 255;
    let actual: u8 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_u16() {
    let value = [
        0 as u8, 5 as u8, '6' as u8, '5' as u8, '5' as u8, '3' as u8, '5' as u8, 0 as u8, 0 as u8,
    ];
    let expected = 65535;
    let actual: u16 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_u32() {
    let value = [
        0 as u8, 5 as u8, '6' as u8, '5' as u8, '5' as u8, '3' as u8, '7' as u8, 0 as u8, 0 as u8,
    ];
    let expected = 65537;
    let actual: u32 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_u64() {
    let value = [
        0 as u8, 7 as u8, '2' as u8, '9' as u8, '6' as u8, '5' as u8, '5' as u8, '3' as u8,
        '7' as u8, 0 as u8, 0 as u8,
    ];
    let expected = 2965537;
    let actual: u64 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_f32() {
    let value = [
        0 as u8, 4 as u8, '1' as u8, '2' as u8, '.' as u8, '9' as u8, 0 as u8, 0 as u8,
    ];
    let expected = 12.9;
    let actual: f32 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_f64() {
    let value = [
        0 as u8, 4 as u8, '1' as u8, '2' as u8, '.' as u8, '9' as u8, 0 as u8, 0 as u8,
    ];
    let expected = 12.9;
    let actual: f64 = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_char() {
    let value = [0 as u8, 1 as u8, 'a' as u8, 0 as u8, 0 as u8];
    let expected = 'a';
    let actual: char = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_str() {
    let value = [
        0 as u8, 4 as u8, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 0 as u8, 0 as u8,
    ];
    let expected = "test";
    let actual: &str = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_string() {
    let value = [
        0 as u8, 4 as u8, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 0 as u8, 0 as u8,
    ];
    let expected = "test".to_string();
    let actual: String = from_bytes(&value).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_deserialize_struct() {
    #[derive(Deserialize)]
    struct TestStruct {
        value: usize,
        name: String,
    }

    let value = [
        0 as u8, 5 as u8, 'v' as u8, 'a' as u8, 'l' as u8, 'u' as u8, 'e' as u8, 0 as u8, 3 as u8,
        '3' as u8, '8' as u8, '3' as u8, 0 as u8, 4 as u8, 'n' as u8, 'a' as u8, 'm' as u8,
        'e' as u8, 0 as u8, 7 as u8, 'a' as u8, 'n' as u8, '-' as u8, 'n' as u8, 'a' as u8,
        'm' as u8, 'e' as u8, 0 as u8, 0 as u8,
    ];

    let actual: TestStruct = from_bytes(&value).unwrap();
    assert_eq!(383, actual.value);
    assert_eq!("an-name".to_string(), actual.name);
}
