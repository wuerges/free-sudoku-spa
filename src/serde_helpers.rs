// serde doesn't impl Serialize/Deserialize for arrays >32 elements.
// Custom helpers convert to/from Vec — error on wrong length.

pub mod u8_81 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(arr: &[u8; 81], s: S) -> Result<S::Ok, S::Error> {
        arr.as_slice().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 81], D::Error> {
        let v = Vec::<u8>::deserialize(d)?;
        if v.len() != 81 {
            return Err(serde::de::Error::invalid_length(v.len(), &"81"));
        }
        let mut arr = [0u8; 81];
        arr.copy_from_slice(&v);
        Ok(arr)
    }
}

pub mod u16_81 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(arr: &[u16; 81], s: S) -> Result<S::Ok, S::Error> {
        arr.as_slice().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u16; 81], D::Error> {
        let v = Vec::<u16>::deserialize(d)?;
        if v.len() != 81 {
            return Err(serde::de::Error::invalid_length(v.len(), &"81"));
        }
        let mut arr = [0u16; 81];
        arr.copy_from_slice(&v);
        Ok(arr)
    }
}
