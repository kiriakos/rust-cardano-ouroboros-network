/**
Forked-off from https://github.com/AndrewWestberg/cncli/ on 2020-11-30
© 2020 Andrew Westberg licensed under Apache-2.0

Re-licensed under GPLv3 or LGPLv3
© 2020 PERLUR Group

SPDX-License-Identifier: GPL-3.0-only OR LGPL-3.0-only

*/

use std::collections::BTreeMap;

use log::debug;
use serde_cbor::{de, ser, Value};

use crate::{Agency, Protocol};

const PROTOCOL_VERSION_1: i128 = 0x01;
const PROTOCOL_VERSION_2: i128 = 0x02;
const PROTOCOL_VERSION_SHELLEY: i128 = 0x03;
const PROTOCOL_VERSION_SHELLEY2: i128 = 0x04;
const PROTOCOL_VERSION_ALLEGRA: i128 = 0x05;
const MIN_PROTOCOL_VERSION: i128 = PROTOCOL_VERSION_ALLEGRA;

const MSG_ACCEPT_VERSION_MSG_ID: i128 = 1;

#[derive(Debug)]
pub enum State {
    Propose,
    Confirm,
    Done,
}

pub struct HandshakeProtocol {
    role: Agency,
    network_magic: u32,
    state: State,
    result: Option<Result<String, String>>,
}

impl HandshakeProtocol {
    pub fn new(network_magic: u32) -> Self {
        HandshakeProtocol {
            role: Agency::Client,
            network_magic,
            state: State::Propose,
            result: None,
        }
    }

    pub fn expect(network_magic: u32) -> Self {
        HandshakeProtocol {
            role: Agency::Server,
            network_magic,
            state: State::Propose,
            result: None,
        }
    }

    // Serialize cbor for MsgProposeVersions
    //
    // Create the byte representation of MsgProposeVersions for sending to the server
    fn msg_propose_versions(&self, network_magic: u32) -> Vec<u8> {
        let mut payload_map: BTreeMap<Value, Value> = BTreeMap::new();
        payload_map.insert(Value::Integer(PROTOCOL_VERSION_1), Value::Integer(network_magic as i128));
        payload_map.insert(Value::Integer(PROTOCOL_VERSION_2), Value::Integer(network_magic as i128));
        payload_map.insert(Value::Integer(PROTOCOL_VERSION_SHELLEY), Value::Integer(network_magic as i128));
        payload_map.insert(Value::Integer(PROTOCOL_VERSION_SHELLEY2), Value::Array(vec![Value::Integer(network_magic as i128), Value::Bool(false)]));
        payload_map.insert(Value::Integer(PROTOCOL_VERSION_ALLEGRA), Value::Array(vec![Value::Integer(network_magic as i128), Value::Bool(false)]));

        let message = Value::Array(vec![
            Value::Integer(0), // message_id
            Value::Map(payload_map)
        ]);

        ser::to_vec_packed(&message).unwrap()
    }

    // Search through the cbor values until we find a Text value.
    fn find_error_message(&self, cbor_value: &Value) -> Result<String, ()> {
        match cbor_value {
            Value::Text(cbor_text) => {
                return Ok(cbor_text.to_owned());
            }
            Value::Array(cbor_array) => {
                for value in cbor_array {
                    let result = self.find_error_message(value);
                    if result.is_ok() {
                        return result;
                    }
                }
            }
            _ => {}
        }
        return Err(());
    }

    fn validate_data(&self, confirm: Value, hex_data: String) -> Result<String, String> {
        let confirm_vec = match &confirm {
            Value::Array(confirm_vec) => { Ok(confirm_vec) }
            _ => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let msg_type = match confirm_vec.get(0) {
            Some(msg_type) => { Ok(msg_type) }
            None => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let _msg_type_int = match msg_type {
            Value::Integer(msg_type_int) => {
                if *msg_type_int == MSG_ACCEPT_VERSION_MSG_ID {
                    Ok(msg_type_int)
                } else {
                    match self.find_error_message(&confirm) {
                        Ok(error_message) => {
                            Err(error_message)
                        }
                        Err(_) => { Err(format!("Unable to parse payload error! {}", hex_data)) }
                    }
                }
            }
            _ => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let accepted_protocol_value = match confirm_vec.get(1) {
            Some(accepted_protocol_value) => { Ok(accepted_protocol_value) }
            None => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let _accepted_protocol = match accepted_protocol_value {
            Value::Integer(accepted_protocol) => {
                if *accepted_protocol < MIN_PROTOCOL_VERSION {
                    Err(format!("Expected protocol version {}, but was {}", MIN_PROTOCOL_VERSION, accepted_protocol))
                } else {
                    Ok(accepted_protocol)
                }
            }
            _ => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let accepted_vec_value = match confirm_vec.get(2) {
            Some(accepted_vec_value) => { Ok(accepted_vec_value) }
            None => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let accepted_vec = match accepted_vec_value {
            Value::Array(accepted_vec) => { Ok(accepted_vec) }
            _ => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let accepted_magic_value = match accepted_vec.get(0) {
            Some(accepted_magic_value) => { Ok(accepted_magic_value) }
            None => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        let _accepted_magic = match accepted_magic_value {
            Value::Integer(accepted_magic) => {
                if *accepted_magic == self.network_magic as i128 {
                    Ok(accepted_magic)
                } else {
                    Err(format!("Expected network magic {}, but was {}", self.network_magic, accepted_magic))
                }
            }
            _ => { Err(format!("Unable to parse payload error! {}", hex_data)) }
        }?;

        return Ok(hex_data);
    }
}

impl Protocol for HandshakeProtocol {
    fn protocol_id(&self) -> u16 {
        let idx: u16 = 0;
        match self.role {
            Agency::Client => idx,
            Agency::Server => idx ^ 0x8000,
            _ => panic!("unknown role"),
        }
    }

    fn result(&self) -> Result<String, String> {
        self.result.clone().unwrap_or(Err("no result".to_string()))
    }

    fn role(&self) -> Agency {
        self.role
    }

    fn agency(&self) -> Agency {
        return match self.state {
            State::Propose => { Agency::Client }
            State::Confirm => { Agency::Server }
            State::Done => { Agency::None }
        };
    }

    fn state(&self) -> String {
        format!("{:?}", self.state)
    }

    fn send_data(&mut self) -> Option<Vec<u8>> {
        debug!("send: {:?}", self.state);
        match self.state {
            State::Propose => {
                let payload = self.msg_propose_versions(self.network_magic);
                self.state = State::Confirm;
                Some(payload)
            }
            State::Confirm => {
                /* TODO: [stub] implement proper negotiation */
                let payload = hex::decode("830105821a2d964a09f4").unwrap();
                self.result = Some(Ok("[stub] confirmed".to_string()));
                self.state = State::Done;
                Some(payload)
            }
            State::Done => panic!("unexpected send"),
        }
    }

    fn receive_data(&mut self, data: Vec<u8>) {
        debug!("recv: {:?}", self.state);
        match self.state {
            State::Propose => {
                self.state = State::Confirm;
            }
            State::Confirm => {
                let confirm: Value = de::from_slice(&data[..]).unwrap();
                debug!("Confirm: {:?}", &confirm);
                self.result = Some(self.validate_data(confirm, hex::encode(data)));
                self.state = State::Done;
            }
            State::Done => panic!("unexpected recv"),
        }
    }
}
