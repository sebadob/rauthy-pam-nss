use crate::error::{Error, ErrorType};
use std::fmt::Debug;

#[inline]
pub fn serialize<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Debug + bincode_next::Encode,
{
    bincode_next::encode_to_vec(value, bincode_next::config::standard()).map_err(|err| {
        Error::new(
            ErrorType::Internal,
            format!("Cannot serialize value: {err:?}"),
        )
    })
}

#[inline]
pub fn deserialize<T>(value: &[u8]) -> Result<T, Error>
where
    T: Debug + bincode_next::Decode<()>,
{
    let (bytes, _) = bincode_next::decode_from_slice(value, bincode_next::config::standard())
        .map_err(|err| {
            Error::new(
                ErrorType::Internal,
                format!("Cannot deserialize value: {err:?}"),
            )
        })?;
    Ok(bytes)
}
