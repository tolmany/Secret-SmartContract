use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// -------------------------------------------------------------------------- //
//                     init, handle, query MESSAGES                           //
// -------------------------------------------------------------------------- //
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    // Maximum size of a reminder message in bytes
    pub max_size: i32,
    pub prng_seed: String, // set a PRNG 'seed' String when the contract is first initialized
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Records a new reminder for the sender
    Record {
        reminder: String,
    },
    // Requests the current reminder for the sender
    Read {},

    // Add a struct to generate a viewing key for a user
    GenerateViewingKey {
        entropy: String,
        padding: Option<String>, // padding is an optional parameter to obfuscate the length of the entropy string
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Gets basic statistics about the use of the contract
    Stats {},

    // when we make a 'Read' query we pass in the address of the querier using their
    // human-friendly secret address and the viewing key string.
    Read { address: HumanAddr, key: String },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::Read { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

// -------------------------------------------------------------------------- //
//                         handle & query RESPONSES                           //
// -------------------------------------------------------------------------- //
// The reponse for the init function will be a default and will be always the same

// Responses from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // Return a status message to let the user know if it succeeded or failed
    Record {
        status: String,
    },
    // Return a status message and the current reminder and its timestamp, if it exists
    Read {
        status: String,
        reminder: Option<String>,
        timestamp: Option<u64>,
    },

    GenerateViewingKey {
        key: ViewingKey,
    },
}

// Responses from query functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    // Return basic statistics about contract
    Stats {
        reminder_count: u64,
    },

    Read {
        status: String,
        reminder: Option<String>,
        timestamp: Option<u64>,
    },
}
