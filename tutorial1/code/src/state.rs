use cosmwasm_std::{ReadonlyStorage, StdError, StdResult, Storage};
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

// -------------------------------------------------------------------------- //
//                              contract state                                //
// -------------------------------------------------------------------------- //
pub static CONFIG_KEY: &[u8] = b"config";

// State information for the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub max_size: u16,
    pub reminder_count: u64,
}

// Reminder message and timestamp
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Reminder {
    pub content: Vec<u8>,
    pub timestamp: u64,
}

// -------------------------------------------------------------------------- //
//                             helper functions                               //
// -------------------------------------------------------------------------- //
// to read and write data to storage

// Serialize a struct using 'bitcode2' and write it to storage using the set() method
pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}

// Retrieve, deserialize, returns the data OR raise an error if the key ws not found
pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    Bincode2::deserialize(
        &storage
            .get(key)
            .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
    )
}

// Same as above function, but returns an 'Option' as a result
pub fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}
