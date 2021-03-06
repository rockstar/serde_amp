extern crate byteorder;
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

mod error;
mod ser;
mod de;

pub use error::Error;
pub use ser::to_amp;
pub use de:: from_bytes;

#[test]
fn test_struct_serialize_deserialize() {
    #[derive(Deserialize, Serialize)]
    struct TestStruct {
        value: usize,
        name: String,
    }

    let data = TestStruct { value: 83, name: "Kilroy".to_string() };
    let result : TestStruct = from_bytes(&to_amp(&data).unwrap()[..]).unwrap();
    assert_eq!(data.value, result.value);
    assert_eq!(data.name, result.name);
}
