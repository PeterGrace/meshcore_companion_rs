#[macro_use]
extern crate tracing;
pub mod commands;
pub mod consts;
pub mod push_events;
pub mod responses;

pub mod contact_mgmt;
mod serial_actor;
mod tests;

use crate::commands::SendingMessageTypes::TxtMsg;
use crate::commands::{
    send_command, GetContacts, MessageEnvelope, Reboot, SendTxtMsg, SendingMessageTypes,
};
pub use crate::commands::{AppStart, Commands};
use crate::consts::*;
use crate::contact_mgmt::{Contact, PublicKey};
use crate::responses::check_internal;
use crate::responses::{
    AckCode, BattAndStorage, ChannelMsg, ChannelMsgV3, Confirmation, ContactMsg, ContactMsgV3,
    DeviceInfo, LoginSuccess, Responses, SelfInfo, TuningParameters,
};
use crate::serial_actor::{serial_loop, SerialFrame};
use crate::Commands::CmdSyncNextMessage;
use lazy_static::lazy_static;
use std::cmp::PartialEq;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum AppError {
    #[error("Misc: {0}")]
    Misc(String),
    #[error("Message Congestion: {0}")]
    Congestion(String),
    #[error("Failed command: {0:#?}")]
    FailedCommand(Commands),
    #[error("Unsupported Command: {0:#?}")]
    UnsupportedCommand(Commands),
    #[error("Entry not found: {0:#?}")]
    NotFound(Commands),
    #[error("Table full: {0:#?}")]
    TableFull(Commands),
    #[error("Bad state: {0:#?}")]
    BadState(Commands),
    #[error("File I/O error: {0:#?}")]
    FileIoError(Commands),
    #[error("Invalid argument: {0:#?}")]
    IllegalArgument(Commands),
}

#[derive(Debug)]
pub struct Companion {
    state: Arc<RwLock<CompanionState>>,
    port: String,
    to_radio_rx: Option<mpsc::Receiver<SerialFrame>>,
    from_radio_tx: mpsc::Sender<SerialFrame>,
}

impl Companion {
    pub async fn get_tuning_parameters(&self) -> Option<TuningParameters> {
        self.state.read().await.tuning_parameters.clone()
    }
}

impl Companion {
    pub async fn get_self_info(&self) -> Option<SelfInfo> {
        self.state.read().await.self_info.clone()
    }
}

#[derive(Debug)]
pub struct CompanionState {
    to_radio_tx: mpsc::Sender<SerialFrame>,
    from_radio_rx: mpsc::Receiver<SerialFrame>,
    contacts: Vec<Contact>,
    pub pending_messages: Vec<MessageTypes>,
    newest_advert_time: u32,
    receive_queue: HashMap<u8, Responses>,
    pub self_info: Option<SelfInfo>,
    device_info: Option<DeviceInfo>,
    pending_acks: HashMap<AckCode, MessageEnvelope>,
    pending_msgs: Vec<SendTxtMsg>,
    battery_millivolts: Option<u16>,
    storage_kb: Option<u32>,
    storage_used_kb: Option<u32>,
    command_queue: VecDeque<Commands>,
    result_queue: VecDeque<Result<Commands, AppError>>,
    exports: HashMap<String, String>,
    tuning_parameters: Option<TuningParameters>,
}


impl Companion {
    pub async fn get_contacts(&self) -> Vec<Contact> {
        self.state.read().await.contacts.clone()
    }
    pub async fn find_contact_by_name(&self, name: &str) -> Option<Contact> {
        let state = self.state.read().await;
        let contacts = state.contacts.clone();
        contacts.iter().find(|c| c.adv_name == name).cloned()
    }
    pub async fn find_contact_by_key_prefix(&self, key: Vec<u8>) -> Option<Contact> {
        let state = self.state.read().await;
        let contacts = state.contacts.clone();
        contacts
            .iter()
            .find(|c| c.public_key.bytes[0..6] == key)
            .cloned()
    }
    pub async fn find_contact_by_full_key(&self, key: Vec<u8>) -> Option<Contact> {
        let state = self.state.read().await;
        let contacts = state.contacts.clone();
        contacts
            .iter()
            .find(|c| c.public_key.bytes[0..32] == key)
            .cloned()
    }

    pub async fn pop_message(&self) -> Option<MessageTypes> {
        let mut state = self.state.write().await;
        state.pending_messages.pop()
    }
    pub async fn peek_result(&self, cmd: Commands) -> Option<Result<Commands, AppError>> {
        let mut state = self.state.write().await;
        if let Some(result) = state.result_queue.clone().iter().find(|r| {
            if let Ok(result_cmd) = r {
                *result_cmd == cmd
            } else {
                false
            }
        }) {
            state.result_queue.retain(|r| r != result);
            Some(result.clone())
        } else {
            None
        }
    }
    pub async fn pop_result(&self) -> Option<Result<Commands, AppError>> {
        let mut state = self.state.write().await;
        state.result_queue.pop_front()
    }
    pub async fn retrieve_export(&self, public_key: PublicKey) -> Option<String> {
        let mut state = self.state.write().await;
        state.exports.remove(&public_key.to_string())
    }
    pub async fn get_public_key(&self) -> Option<PublicKey> {
        let state = self.state.read().await;
        if let Some(self_info) = &state.self_info {
            Some(self_info.clone().public_key)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessageTypes {
    ChannelMsg(ChannelMsg),
    ChannelMsgV3(ChannelMsgV3),
    ContactMsg(ContactMsg),
    ContactMsgV3(ContactMsgV3),
}

impl Companion {
    pub fn new(port: &str) -> Self {
        let (to_radio_tx, to_radio_rx) = mpsc::channel(consts::MPSC_BUFFER_DEPTH);
        let (from_radio_tx, from_radio_rx) = mpsc::channel(consts::MPSC_BUFFER_DEPTH);
        let state = Arc::new(RwLock::new(CompanionState {
            to_radio_tx,
            from_radio_rx,
            contacts: vec![],
            pending_messages: vec![],
            newest_advert_time: 0,
            receive_queue: HashMap::new(),
            self_info: None,
            device_info: None,
            pending_acks: HashMap::new(),
            pending_msgs: vec![],
            battery_millivolts: None,
            storage_kb: None,
            storage_used_kb: None,
            command_queue: VecDeque::new(),
            result_queue: VecDeque::new(),
            exports: HashMap::new(),
            tuning_parameters: None,
        }));
        Companion {
            port: port.to_string(),
            to_radio_rx: Some(to_radio_rx),
            from_radio_tx,
            state,
        }
    }
    pub async fn start(&mut self) -> Result<(), AppError> {
        let port = self.port.clone();
        let from_radio_tx = self.from_radio_tx.clone();
        let mut to_radio_rx = self
            .to_radio_rx
            .take()
            .ok_or_else(|| AppError::Misc("Listener already started".to_string()))?;

        tokio::task::Builder::new()
            .name("serial-loop")
            .spawn(async move {
                serial_loop(port, &mut to_radio_rx, &from_radio_tx).await;
            });
        let state_handle = self.state.clone();
        tokio::task::Builder::new()
            .name("background-processor")
            .spawn(async move {
                info!("Background processor started.");
                loop {
                    // We pass the clone into our internal function
                    if let Err(e) = check_internal(state_handle.clone()).await {
                        error!("Background processor encountered an error: {}", e);
                        // Depending on the error, you might want to break or continue
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                    // Small sleep to prevent tight-looping if no messages are arriving
                    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                }
            });

        Ok(())
    }
    pub async fn command(&self, cmd: Commands) -> Result<(), AppError> {
        send_command(&self.state, cmd).await
    }
}
#[derive(Clone)]
pub struct HexData {
    bytes: Vec<u8>,
}
impl fmt::Display for HexData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
impl fmt::Debug for HexData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

pub struct InferredAdvert {
    code: u8,
    something: u8,
    public_key: PublicKey,
    other_stuff: Vec<u8>,
}
impl InferredAdvert {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut something = [0u8; 1];
        cursor.read_exact(&mut something).unwrap();
        let mut public_key = [0u8; 32];
        cursor.read_exact(&mut public_key).unwrap();
        let mut other_stuff = vec![];
        cursor.read_to_end(&mut other_stuff).unwrap();
        Self {
            code: code[0],
            something: something[0],
            public_key: PublicKey::from_bytes(public_key),
            other_stuff,
        }
    }
}

pub fn string_to_bytes<const N: usize>(s: &str) -> [u8; N] {
    let bytes = s.as_bytes();
    let mut result = [0u8; N];
    let len = bytes.len().min(N);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}
