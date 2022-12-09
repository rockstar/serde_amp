mod de;
mod error;
mod ser;

pub use de::from_bytes;
pub use error::Error;
pub use ser::to_amp;

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn test_struct_serialize_deserialize() {
        #[derive(Deserialize, Serialize)]
        struct TestStruct {
            value: usize,
            name: String,
        }

        let data = TestStruct {
            value: 83,
            name: "Kilroy".to_string(),
        };
        let result: TestStruct = from_bytes(&to_amp(&data).unwrap()[..]).unwrap();
        assert_eq!(data.value, result.value);
        assert_eq!(data.name, result.name);
    }
}
