use crate::error::{Error, ErrorType};
use std::fmt::Debug;

#[inline]
pub fn serialize<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Debug + serde::Serialize,
{
    bincode::serde::encode_to_vec(value, bincode::config::standard()).map_err(|err| {
        Error::new(
            ErrorType::Internal,
            format!("Cannot serialize value: {err:?}"),
        )
    })
}

// #[inline]
// pub fn deserialize<T>(value: &[u8]) -> Result<T, Error>
// where
//     T: Debug + serde::de::DeserializeOwned,
// {
//     let (bytes, _) = bincode::serde::decode_from_slice(value, bincode::config::standard())
//         .map_err(|err| {
//             Error::new(
//                 ErrorType::Internal,
//                 format!("Cannot deserialize value: {:?}", err),
//             )
//         })?;
//     Ok(bytes)
// }
