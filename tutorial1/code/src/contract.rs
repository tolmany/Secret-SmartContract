use crate::msg::{HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use crate::state::{
    load, may_load, read_viewing_key, save, write_viewing_key, Reminder, State, CONFIG_KEY,
};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    QueryResult, StdError, StdResult, Storage,
};
use secret_toolkit::crypto::sha_256;
use std::convert::TryFrom;

// -------------------------------------------------------------------------- //
//                                  initialize                                //
// -------------------------------------------------------------------------- //

// limit the max message size to values in 1..65535
fn valid_max_size(val: i32) -> Option<u16> {
    if val < 1 {
        None
    } else {
        u16::try_from(val).ok()
    }
}

// Init function
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // Check whether the reminder's maximum size exceeds its upper bound, i.e., a u16 type
    let max_size = match valid_max_size(msg.max_size) {
        Some(v) => v,
        None => {
            return Err(StdError::generic_err(
                "Invalid max_size. Must be in the range of 1..65535.",
            ))
        }
    };

    // New instantiation of the state function
    // store the 'seed' when the contract is initialized
    let config = State {
        max_size,
        reminder_count: 0_u64,
        prng_seed: sha_256(base64::encode(msg.prng_seed).as_bytes()).to_vec(), // encode the 'seed' as a hashed Base64
    };

    // Save the state function and send it to storage
    save(&mut deps.storage, CONFIG_KEY, &config)?;

    // Return a default 'InitResponse'
    Ok(InitResponse::default())
}

//  -------------------------------------------------------------------------- //
//                                     handle                                  //
//  -------------------------------------------------------------------------- //

// Record the message if you can and abort it if message exceeds allowed size
fn try_record<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    reminder: String,
) -> StdResult<HandleResponse> {
    let status: String;
    let reminder = reminder.as_bytes();

    // retrieve the config state from storage
    let mut config: State = load(&mut deps.storage, CONFIG_KEY)?;

    if reminder.len() > config.max_size.into() {
        // if reminder content is too long, set status message and do nothing else
        status = String::from("Message is too long. Reminder not recorded.");
    } else {
        // get the canonical address of sender
        let sender_address = deps.api.canonical_address(&env.message.sender)?;

        // create the reminder struct containing content string and timestamp
        let stored_reminder = Reminder {
            content: reminder.to_vec(),
            timestamp: env.block.time,
        };

        // save the reminder using a byte vector representation of the sender's address as the key
        save(
            &mut deps.storage,
            &sender_address.as_slice().to_vec(),
            &stored_reminder,
        )?;

        // increment the reminder_count
        config.reminder_count += 1;
        save(&mut deps.storage, CONFIG_KEY, &config)?;

        // set the status message
        status = String::from("Reminder recorded!");
    }

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Record { status })?),
    })
}

// Try and read the message if there is one
fn try_read<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let status: String;
    let mut reminder: Option<String> = None;
    let mut timestamp: Option<u64> = None;

    let sender_address = deps.api.canonical_address(&env.message.sender)?;

    // read the reminder from storage
    let result: Option<Reminder> = may_load(&mut deps.storage, &sender_address.as_slice().to_vec())
        .ok()
        .unwrap();
    match result {
        // set all response field values
        Some(stored_reminder) => {
            status = String::from("Reminder found.");
            reminder = String::from_utf8(stored_reminder.content).ok();
            timestamp = Some(stored_reminder.timestamp);
        }
        // unless there's an error
        None => {
            status = String::from("Reminder not found.");
        }
    };

    // Return a HandleResponse with status message, reminder, and timestamp included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Read {
            status,
            reminder,
            timestamp,
        })?),
    })
}

pub fn try_generate_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let config: State = load(&mut deps.storage, CONFIG_KEY)?;
    let prng_seed = config.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenerateViewingKey { key })?),
    })
}

// 'handle' function
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Record { reminder } => try_record(deps, env, reminder),
        HandleMsg::Read {} => try_read(deps, env),
        HandleMsg::GenerateViewingKey { entropy, .. } => {
            try_generate_viewing_key(deps, env, entropy)
        }
    }
}

// -------------------------------------------------------------------------- //
//                                    query                                   //
// -------------------------------------------------------------------------- //
// A query function to return the binary encoded 'Stats' struct.
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Stats {} => query_stats(deps),
        _ => authenticated_queries(deps, msg), // deal with all authenticated queries
    }
}

fn query_stats<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    // retrieve the config state from storage
    let config: State = load(&deps.storage, CONFIG_KEY)?;
    to_binary(&QueryAnswer::Stats {
        reminder_count: config.reminder_count,
    })
}

// Check whether the correct viewing key has been sent for a given :
// --> if the key matches, then we can handle the 'read' query
// --> if the viewing key does not match or was not set, then we return an unauthorized error
fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let expected_key = read_viewing_key(&deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                QueryMsg::Read { address, .. } => query_read(&deps, &address),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Err(StdError::unauthorized())
}

// Similarly to the try_read function, the query_read function uses the sender address to read and return the reminder - withouht paying any SCRT tokens!
fn query_read<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
) -> StdResult<Binary> {
    let status: String;
    let mut reminder: Option<String> = None;
    let mut timestamp: Option<u64> = None;

    let sender_address = deps.api.canonical_address(&address)?;

    // read the reminder from storage
    let result: Option<Reminder> = may_load(&deps.storage, &sender_address.as_slice().to_vec())
        .ok()
        .unwrap();
    match result {
        // set all response field values
        Some(stored_reminder) => {
            status = String::from("Reminder found.");
            reminder = String::from_utf8(stored_reminder.content).ok();
            timestamp = Some(stored_reminder.timestamp);
        }
        // unless there's an error
        None => {
            status = String::from("Reminder not found.");
        }
    };

    to_binary(&QueryAnswer::Read {
        status,
        reminder,
        timestamp,
    })
}
