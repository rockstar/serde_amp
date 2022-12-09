use std::convert::TryInto;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{ser, Serialize};

use error::{Error, Result};

fn usize_to_bytes(integer: usize) -> [u8; 2] {
    if integer > std::u16::MAX as usize {
        panic!("Key length in response too long");
    }

    let mut bytearray = Vec::with_capacity(2);
    bytearray.write_u16::<BigEndian>(integer as u16).unwrap();
    match bytearray.try_into() {
        Ok(value) => value,
        Err(err) => panic!("{:?}", err),
    }
}

struct Serializer {
    // Due to the way that serde serializes, we must keep a "start" index
    // for where we should insert the byte length. This is kept as a stack,
    // as we may have multiple markers.
    byte_indexes: Vec<usize>,

    output: Vec<u8>,
}

impl Serializer {
    // Amp requires termination with bytes 0x00 0x00. serde doesn't *seem*
    // to have a `end`-type call for termination. This must be called
    // explicitly.
    fn end(&mut self) {
        self.output.extend(vec![0_u8, 0_u8]);
    }
}

pub fn to_amp<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut serializer = Serializer {
        byte_indexes: vec![],
        output: vec![],
    };
    value.serialize(&mut serializer)?;
    serializer.end();
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if v {
            self.serialize_str("True")
        } else {
            self.serialize_str("False")
        }
    }
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }
    fn serialize_str(self, v: &str) -> Result<()> {
        let bytes = v.as_bytes();
        self.output.extend(usize_to_bytes(bytes.len()).iter());
        self.output.extend(v.as_bytes());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v as f64)
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        unimplemented!();
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }
    fn serialize_unit(self) -> Result<()> {
        unimplemented!();
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.byte_indexes.push(self.output.len());
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!();
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!();
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        let index = self.byte_indexes.pop().unwrap();

        let count = self.output.len() - index;
        let bytes = usize_to_bytes(count);

        self.output.insert(index, bytes[0]);
        self.output.insert(index + 1, bytes[1]);

        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
    }
}

#[test]
fn test_serialize_bool_true() {
    let expected = vec![
        0 as u8, 4 as u8, 'T' as u8, 'r' as u8, 'u' as u8, 'e' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&true).unwrap());
}
#[test]
fn test_serialize_bool_false() {
    let expected = vec![
        0 as u8, 5 as u8, 'F' as u8, 'a' as u8, 'l' as u8, 's' as u8, 'e' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&false).unwrap());
}
#[test]
fn test_serialize_char() {
    let an_char = 'X';
    let expected = vec![0 as u8, 1 as u8, 'X' as u8, 0 as u8, 0 as u8];
    assert_eq!(expected, to_amp(&an_char).unwrap());
}
#[test]
fn test_serialize_str() {
    let an_str = "An string";
    let expected = vec![
        0 as u8, 9 as u8, 'A' as u8, 'n' as u8, ' ' as u8, 's' as u8, 't' as u8, 'r' as u8,
        'i' as u8, 'n' as u8, 'g' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&an_str).unwrap());
}

#[test]
fn test_serialize_u8() {
    let number: u8 = 10;
    let expected = vec![0 as u8, 2 as u8, '1' as u8, '0' as u8, 0 as u8, 0 as u8];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_u16() {
    let number: u16 = 100;
    let expected = vec![
        0 as u8, 3 as u8, '1' as u8, '0' as u8, '0' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_u32() {
    let number: u32 = 1000;
    let expected = vec![
        0 as u8, 4 as u8, '1' as u8, '0' as u8, '0' as u8, '0' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_u64() {
    let number: u64 = 10000;
    let expected = vec![
        0 as u8, 5 as u8, '1' as u8, '0' as u8, '0' as u8, '0' as u8, '0' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}

#[test]
fn test_serialize_i8() {
    let number: i8 = -10;
    let expected = vec![
        0 as u8, 3 as u8, '-' as u8, '1' as u8, '0' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_i16() {
    let number: i16 = -100;
    let expected = vec![
        0 as u8, 4 as u8, '-' as u8, '1' as u8, '0' as u8, '0' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_i32() {
    let number: i32 = -1000;
    let expected = vec![
        0 as u8, 5 as u8, '-' as u8, '1' as u8, '0' as u8, '0' as u8, '0' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_i64() {
    let number: i64 = -10000;
    let expected = vec![
        0 as u8, 6 as u8, '-' as u8, '1' as u8, '0' as u8, '0' as u8, '0' as u8, '0' as u8,
        0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}

#[test]
fn test_serialize_f32() {
    let number: f32 = 1.5;
    let expected = vec![
        0 as u8, 3 as u8, '1' as u8, '.' as u8, '5' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}
#[test]
fn test_serialize_f64() {
    let number: f64 = 10.5;
    let expected = vec![
        0 as u8, 4 as u8, '1' as u8, '0' as u8, '.' as u8, '5' as u8, 0 as u8, 0 as u8,
    ];
    assert_eq!(expected, to_amp(&number).unwrap());
}

#[test]
fn test_some() {
    let expected: Vec<u8> = vec![0 as u8, 1 as u8, '1' as u8, 0 as u8, 0 as u8];
    let value: Option<u8> = Some(1);
    assert_eq!(expected, to_amp(&value).unwrap());
}

#[test]
fn test_struct() {
    let expected = vec![
        0 as u8, 5 as u8, 'v' as u8, 'a' as u8, 'l' as u8, 'u' as u8, 'e' as u8, 0 as u8, 2 as u8,
        '1' as u8, '0' as u8, 0 as u8, 6 as u8, 'n' as u8, 'e' as u8, 's' as u8, 't' as u8,
        'e' as u8, 'd' as u8, 0 as u8, 5 as u8, 'i' as u8, 'n' as u8, 'n' as u8, 'e' as u8,
        'r' as u8, 0 as u8, 1 as u8, '1' as u8, 0 as u8, 0 as u8,
    ];

    #[derive(Serialize)]
    struct NestedStruct {
        inner: usize,
    }

    #[derive(Serialize)]
    struct TestStruct {
        value: usize,
        nested: NestedStruct,
    }

    let value = TestStruct {
        value: 10,
        nested: NestedStruct { inner: 1 },
    };
    assert_eq!(expected, to_amp(&value).unwrap());
}

#[test]
fn test_sequence() {
    let expected = vec![
        0 as u8, 8 as u8, 0 as u8, 2 as u8, '1' as u8, '0' as u8, 0 as u8, 2 as u8, '1' as u8,
        '1' as u8, 0 as u8, 0 as u8,
    ];

    let value = vec![10, 11];
    assert_eq!(expected, to_amp(&value).unwrap());
}
