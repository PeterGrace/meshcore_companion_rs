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
pub use crate::commands::{AppStart, Commands};
use crate::commands::{GetContacts, MessageEnvelope, Reboot, SendTxtMsg, SendingMessageTypes};
use crate::consts::*;
use crate::contact_mgmt::{Contact, PublicKey};
use crate::responses::{AckCode, BattAndStorage, ChannelMsg, ChannelMsgV3, Confirmation, ContactMsg, ContactMsgV3, DeviceInfo, LoginSuccess, Responses, SelfInfo};
use crate::serial_actor::{serial_loop, SerialFrame};
use crate::Commands::CmdSyncNextMessage;
use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Debug, Error)]
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
    pub async fn get_self_info(&self) -> SelfInfo {
        self.state.read().await.self_info.clone().unwrap()
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
    exports: HashMap<String, String>
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
        contacts.iter().find(|c| c.public_key.bytes[0..6] == key).cloned()
    }
    pub async fn find_contact_by_full_key(&self, key: Vec<u8>) -> Option<Contact> {
        let state = self.state.read().await;
        let contacts = state.contacts.clone();
        contacts.iter().find(|c| c.public_key.bytes[0..32] == key).cloned()
    }


    pub async fn pop_message(&self) -> Option<MessageTypes> {
        let mut state = self.state.write().await;
        state.pending_messages.pop()
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
            exports: HashMap::new()
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
pub async fn send_command(
    state: &Arc<RwLock<CompanionState>>,
    cmd: Commands,
) -> Result<(), AppError> {
    let tx = state.write().await.to_radio_tx.clone();
    match cmd {
        Commands::CmdResetPath(ref pubkey) => {
            let mut data: Vec<u8> = vec![CMD_RESET_PATH];
            let pubkey_bytes = pubkey.bytes;
            data.extend_from_slice(&pubkey_bytes);
            let frame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdSetRadioParams(ref radioparams) => {
            //    if (freq >= 300000 && freq <= 2500000 && sf >= 5 && sf <= 12 && cr >= 5 && cr <= 8 && bw >= 7000 &&
            //         bw <= 500000) {
            let data: Vec<u8> = radioparams.to_frame();
            let frame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdSetAdvertLatLon(ref coords) => {
            let mut data: Vec<u8> = vec![CMD_SET_ADVERT_LATLON];
            let coords_bytes = coords.to_frame();
            data.extend_from_slice(&coords_bytes);
            let frame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdSetAdvertName(ref name) => {
            let mut data: Vec<u8> = vec![CMD_SET_ADVERT_NAME];
            let name_bytes = name.as_bytes();
            data.extend_from_slice(&name_bytes);
            let frame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdRemoveContact(key) => {
            let mut data: Vec<u8> = vec![CMD_REMOVE_CONTACT];
            let contact_bytes = key.bytes;
            data.extend_from_slice(&contact_bytes);
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdExportContact(None) => {
            let data: Vec<u8> = vec![CMD_EXPORT_CONTACT];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));

            Ok(())
        }
        Commands::CmdExportContact(Some(contact)) => {
            let mut data: Vec<u8> = vec![CMD_EXPORT_CONTACT];
            let contact_bytes = contact.bytes;
            data.extend_from_slice(&contact_bytes);
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));

            Ok(())
        }
        Commands::CmdSetDeviceTime => {
            let mut data: Vec<u8> = vec![CMD_SET_DEVICE_TIME];
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let timestamp_bytes = timestamp.to_le_bytes();
            data.extend_from_slice(&timestamp_bytes);
            let frame: SerialFrame = SerialFrame::from_data(data);
            info!("Setting device time to {}", timestamp);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdGetDeviceTime => {
            let data: Vec<u8> = vec![CMD_GET_DEVICE_TIME];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdGetBattAndStorage => {
            let data: Vec<u8> = vec![CMD_GET_BATT_AND_STORAGE];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSendSelfAdvert(ref advert_mode) => {
            let data: Vec<u8> = vec![CMD_SEND_SELF_ADVERT, advert_mode.clone() as u8];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())

        }
        Commands::CmdReboot => {
            let data: Vec<u8> = vec![0x13, 0x72, 0x65, 0x62, 0x6f, 0x6f, 0x74];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdAppStart(app) => {
            // Send command
            let data: Vec<u8> = vec![
                consts::CMD_APP_START,
                0x03,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x01,
            ];

            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdDeviceQuery(app) => {
            // Send command
            let data = vec![consts::CMD_DEVICE_QEURY, app.app_target_ver];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSyncNextMessage => {
            let data = vec![10u8];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdGetContacts(payload) => {
            let mut data = vec![payload.code];
            let since = u32::to_le_bytes(payload.since.unwrap_or(0));
            data.extend_from_slice(&since);
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSendTxtMsg(msg) => {
            let pending_msgs = state.write().await.pending_msgs.clone();
            if pending_msgs.len() > 0 {
                return Err(AppError::Congestion(
                    "Messages still awaiting expected ack code".to_string(),
                ));
            }
            state.write().await.pending_msgs.push(msg.clone());
            let data = msg.to_frame();
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSendChannelTxtMsg(ref msg) => {
            let data = msg.to_frame();
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
        Commands::CmdSendLogin(login) => {
            let data = login.to_frame();
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        _ => todo!(),
    }
}
#[instrument(skip(state))]
async fn check_internal(state: Arc<RwLock<CompanionState>>) -> Result<(), AppError> {


    //region check for inbound radio messages
    let mut messages = vec![];
    {
        let mut lock = state.write().await;
        while let Ok(msg) = lock.from_radio_rx.try_recv() {
            messages.push(msg);
        }
    }
    for msg in messages {
        let frame = msg.frame;
        match frame[0] {
            consts::PUSH_CODE_LOGIN_FAIL => {
                error!("Login failed.");
            }
            consts::PUSH_CODE_LOGIN_SUCCESS => {
                let lock = state.read().await;
                let login_success = LoginSuccess::from_frame(&frame);
                let pubkey = login_success.pub_key_prefix.to_vec();
                info!("Login successful to {:?}",pubkey);
            }
            consts::PUSH_CODE_SEND_CONFIRMED => {
                info!("Received send confirmed, ack received.");
                let confirmation = Confirmation::from_frame(&frame);
                {
                    let mut state = state.write().await;
                    if state.pending_acks.contains_key(&confirmation.ack_code) {
                        info!("Received send confirmation: {confirmation:?}");
                        state.pending_acks.remove(&confirmation.ack_code);
                    } else {
                        warn!("Received send confirmation for unknown ack code: {confirmation:?}");
                    }
                }
            }
            consts::PUSH_CODE_ADVERT => {
                info!("Received new advert, requesting contact sync.");
                let get_contacts = GetContacts {
                    code: CMD_GET_CONTACTS,
                    since: Some(state.read().await.newest_advert_time),
                };
                let _ = send_command(&state, Commands::CmdGetContacts(get_contacts)).await;
            }
            consts::PUSH_CODE_MSG_WAITING => {
                debug!("Received Message Waiting Indicator");
                let _ = send_command(&state, Commands::CmdSyncNextMessage).await;
            }
            consts::PUSH_CODE_PATH_UPDATED => {
                info!("Received updated path for a contact, requesting full contact sync.");
                let get_contacts = GetContacts {
                    code: CMD_GET_CONTACTS,
                    since: Some(state.read().await.newest_advert_time),
                };
                let _ = send_command(&state, Commands::CmdGetContacts(get_contacts)).await;
            }
            consts::PUSH_CODE_LOG_RX_DATA => {
                let snr = i32::from_le_bytes([frame[1], frame[2], frame[3], frame[4]]);
                let rssi = frame[5];
                let data = frame[6..].to_vec();
                debug!("Received log rx data: snr: {snr}, rssi: {rssi}, data: {data:?}");
            }
            consts::RESP_CODE_CURR_TIME => {
                let curr_time = u32::from_le_bytes([frame[1], frame[2], frame[3], frame[4]]);
                info!("Received radio's current time: {curr_time}");
            }
            consts::RESP_CODE_OK => {
                {
                    let mut lock = state.write().await;
                    if let Some(cmd) = lock.command_queue.pop_front() {
                        lock.result_queue.push_back(Ok(cmd));
                    } else {
                        error!("Received OK response, but no commands to associate it with.");
                    }
                }
            }
            consts::RESP_CODE_ERR => {
                {
                    let err_code = frame[1];
                    let mut lock = state.write().await;
                    if let Some(cmd) = lock.command_queue.pop_front() {
                        let err = match err_code {
                            consts::ERR_CODE_UNSUPPORTED_CMD => AppError::UnsupportedCommand(cmd),
                            consts::ERR_CODE_NOT_FOUND => AppError::NotFound(cmd),
                            consts::ERR_CODE_TABLE_FULL => AppError::TableFull(cmd),
                            consts::ERR_CODE_BAD_STATE => AppError::BadState(cmd),
                            consts::ERR_CODE_FILE_IO_ERROR => AppError::FileIoError(cmd),
                            consts::ERR_CODE_ILLEGAL_ARG => AppError::IllegalArgument(cmd),
                            _ => AppError::FailedCommand(cmd)
                        };
                        lock.result_queue.push_back(Err(err));
                    } else {
                        error!("Received Err response, but no commands to associate it with.");
                    }
                }
            }
            consts::RESP_CODE_EXPORT_CONTACT => {
                info!("Frame len is {}", frame.len());
                let hexdata: HexData =  HexData { bytes: frame[1..].to_vec()};
                let inferred: InferredAdvert = InferredAdvert::from_frame(&hexdata.bytes);

                let url = format!("meshcore://{}", hexdata);
                info!("Received export contact response: {url}");
                state.write().await.exports.insert(inferred.public_key.to_string(), url);
            }
            consts::RESP_CODE_SENT => {
                let tx_type: u8 = frame[1];
                let exp_ack: AckCode = frame[2..6].try_into().map(AckCode).unwrap();
                let suggested_timeout = frame[6..10].try_into().map(u32::from_le_bytes).unwrap();

                {
                    let mut state = state.write().await;
                    if let Some(msg) = state.pending_msgs.pop() {
                        let mut msg_timeout = msg.clone();
                        msg_timeout.timeout = Some(suggested_timeout);
                        let envelope = MessageEnvelope {
                            msg: TxtMsg(msg_timeout),
                            last_attempt_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                        };
                        state.pending_acks.insert(exp_ack.clone(), envelope);
                        info!("Assigning {exp_ack} ack code for msg {msg:?}");
                    } else {
                        info!("Received ack for message we aren't tracking.  Maybe a login.");
                    }
                }

                match tx_type {
                    0 => {
                        info!(
                            "Message sent via Flood Routing.  Ack expected: {exp_ack:?}, suggested timeout is {suggested_timeout}ms"
                        );
                    }
                    1 => {
                        info!(
                            "Message sent via Direct Routing.  Ack expected: {exp_ack:?}, suggested timeout is {suggested_timeout}ms"
                        );
                    }
                    _ => {
                        info!(
                            "Message sent via unknown type {tx_type}  Ack expected: {exp_ack:?}, suggested timeout is {suggested_timeout}ms"
                        );
                    }
                }
            }
            consts::RESP_CODE_SELF_INFO => {
                let self_info = SelfInfo::from_frame(&frame);
                debug!("Received self info response: {self_info:#?}");
                state.write().await.self_info = Some(self_info);
            }
            consts::RESP_CODE_DEVICE_INFO => {
                let device_info = DeviceInfo::from_frame(&frame);
                debug!("Received device info response: {device_info:#?}");
                state.write().await.device_info = Some(device_info);
            }
            consts::RESP_CODE_CONTACTS_START => {
                let count = frame[1];
                debug!("Received contacts start, {count} contacts follow.");
            }
            consts::RESP_CODE_CONTACT => {
                let contact = Contact::from_frame(&frame);
                debug!("Received contact: {contact:?}");
                state.write().await.contacts.push(contact);
            }
            consts::RESP_CODE_END_OF_CONTACTS => {
                let last_modified = u32::from_le_bytes([frame[1], frame[2], frame[3], frame[4]]);
                info!("Received end of contacts, newest advert time: {last_modified}");
                state.write().await.newest_advert_time = last_modified;
            }
            consts::RESP_CODE_CONTACT_MSG_RECV => {
                let contact_msg = ContactMsg::from_frame(&frame);
                debug!("Received contact message: {contact_msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ContactMsg(contact_msg));
                let _ = send_command(&state, Commands::CmdSyncNextMessage).await;
            }
            consts::RESP_CODE_CONTACT_MSG_RECV_V3 => {
                let contact_msg = ContactMsgV3::from_frame(&frame);
                debug!("Received channel message: {contact_msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ContactMsgV3(contact_msg));
                let _ = send_command(&state, Commands::CmdSyncNextMessage).await;
            }
            consts::RESP_CODE_CHANNEL_MSG_RECV => {
                let msg = ChannelMsg::from_frame(&frame);
                debug!("Received channel message: {msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ChannelMsg(msg));
                let _ = send_command(&state, Commands::CmdSyncNextMessage).await;
            }
            consts::RESP_CODE_CHANNEL_MSG_RECV_V3 => {
                let msg = ChannelMsgV3::from_frame(&frame);
                debug!("Received channel message: {msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ChannelMsgV3(msg));
                let _ = send_command(&state, Commands::CmdSyncNextMessage).await;
            }
            consts::RESP_CODE_NO_MORE_MESSAGES => {
                debug!("No more messages to sync.");
            }
            consts::RESP_CODE_BATT_AND_STORAGE => {
                let msg = BattAndStorage::from_frame(&frame);
                {
                    let mut lock = state.write().await;
                    lock.battery_millivolts = Some(msg.milli_volts.clone());
                    lock.storage_kb = Some(msg.total_kb.clone());
                    lock.storage_used_kb = Some(msg.used_kb.clone());
                }
                debug!("Received battery and storage info: {msg:#?}");
            }
            _ => {
                warn!(
                    "unimplemented response code: {:02x} {:02x?}",
                    frame[0], frame
                );
            }
        }
    }
    //endregion

    //region check for messages that require re-delivery attempts
    let mut pending_sends = vec![];
    {
        let mut lock = state.read().await;
        for (k, v) in lock.pending_acks.iter() {
            pending_sends.push((k.clone(), v.clone()));
        }
    }
    for (ack_code, _) in pending_sends {
        let mut lock = state.write().await;
        if let Some(mut envelope) = lock.pending_acks.remove(&ack_code) {
            match envelope.clone().msg {
                SendingMessageTypes::TxtMsg(prev_msg) => {
                    let mut msg = prev_msg;
                    if msg.attempt > 2 {
                        warn!("Message {msg:?}, failed to receive ack after 3 attempts.");
                    } else {
                        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                        if current_time - envelope.last_attempt_timestamp > msg.timeout.unwrap() as u128 {
                            msg.attempt += 1;
                            info!("Resending message {msg:?}");
                            drop(lock);
                            let _ = send_command(&state, Commands::CmdSendTxtMsg(msg.clone())).await;
                        } else {
                            lock.pending_acks.insert(ack_code, envelope);
                        }
                    }
                }
                SendingMessageTypes::ChannelMsg(msg) => {
                    todo!()
                }
            }
        } else {
            warn!("msg existed in first sweep but not second, this should not happen.")
        }
    }
    //endregion

    Ok(())
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
}impl fmt::Debug for HexData {
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
    other_stuff: Vec<u8>
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
            other_stuff
        }
    }
}