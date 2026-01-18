#[macro_use]
extern crate tracing;
pub mod commands;
pub mod consts;
pub mod push_events;
pub mod responses;

mod contact_mgmt;
mod serial_actor;
mod tests;

pub use crate::commands::{AppStart, Commands};
use crate::commands::{GetContacts, Reboot};
use crate::consts::CMD_GET_CONTACTS;
use crate::contact_mgmt::Contact;
use crate::responses::{
    ChannelMsg, ChannelMsgV3, ContactMsg, ContactMsgV3, DeviceInfo, Responses, SelfInfo,
};
use crate::serial_actor::{serial_loop, SerialFrame};
use crate::Commands::CmdSyncNextMessage;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Misc: {0}")]
    Misc(String),
}

#[derive(Debug)]
pub struct Companion {
    state: Arc<RwLock<CompanionState>>,
    port: String,
}

#[derive(Debug)]
pub struct CompanionState {
    to_radio_tx: mpsc::Sender<SerialFrame>,
    to_radio_rx: Option<mpsc::Receiver<SerialFrame>>,
    from_radio_tx: mpsc::Sender<SerialFrame>,
    from_radio_rx: mpsc::Receiver<SerialFrame>,
    contacts: Vec<Contact>,
    pub pending_messages: Vec<MessageTypes>,
    newest_advert_time: u32,
    receive_queue: HashMap<u8, Responses>,
    self_info: Option<SelfInfo>,
    device_info: Option<DeviceInfo>,
}

impl Companion {
    pub async fn find_contact(&self, name: &str) -> Option<Contact> {
        let state = self.state.read().await;
        state.contacts.iter().find(|c| c.adv_name == name).cloned()
    }

    pub async fn pop_message(&self) -> Option<MessageTypes> {
        let mut state = self.state.write().await;
        state.pending_messages.pop()
    }
}

#[derive(Debug)]
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
            to_radio_rx: None,
            from_radio_tx,
            from_radio_rx,
            contacts: vec![],
            pending_messages: vec![],
            newest_advert_time: 0,
            receive_queue: HashMap::new(),
            self_info: None,
            device_info: None,
        }));
        Companion {
            port: port.to_string(),
            state,
        }
    }
    pub async fn start(&mut self) -> Result<(), AppError> {
        let port = self.port.clone();
        let from_radio_tx = self.state.read().await.from_radio_tx.clone();
        let mut to_radio_rx = self
            .state
            .write()
            .await
            .to_radio_rx
            .take()
            .ok_or_else(|| AppError::Misc("Listener already started".to_string()))?;

        tokio::spawn(async move {
            serial_loop(port, &mut to_radio_rx, &from_radio_tx).await;
        });
        let state_handle = self.state.clone();
        tokio::spawn(async move {
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
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        Ok(())
    }
    pub async fn command(&self, cmd: Commands) -> Result<(), AppError> {
        send_command(&self.state.read().await.to_radio_tx, cmd).await
    }
}
pub async fn send_command(tx: &mpsc::Sender<SerialFrame>, cmd: Commands) -> Result<(), AppError> {
    match cmd {
        Commands::CmdReboot => {
            let data: Vec<u8> = vec![0x13, 0x72, 0x65, 0x62, 0x6f, 0x6f, 0x74];
            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
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

            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdDeviceQuery(app) => {
            // Send command
            let data = vec![consts::CMD_DEVICE_QEURY, app.app_target_ver];
            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSyncNextMessage => {
            let data = vec![10u8];
            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdGetContacts(payload) => {
            let mut data = vec![payload.code];
            let since = u32::to_le_bytes(payload.since.unwrap_or(0));
            data.extend_from_slice(&since);
            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSendTxtMsg(msg) => {
            let data = msg.to_frame();
            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        Commands::CmdSendChannelTxtMsg(msg) => {
            let data = msg.to_frame();
            let frame: SerialFrame = SerialFrame {
                delimiter: consts::SERIAL_OUTBOUND,
                frame_length: data.len() as u16,
                frame: data,
            };
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            Ok(())
        }
        _ => todo!(),
    }
}
async fn check_internal(state: Arc<RwLock<CompanionState>>) -> Result<(), AppError> {
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
            consts::RESP_CODE_OK => {
                info!("Received OK response.");
            }
            consts::RESP_CODE_ERR => {
                error!("Received error response.");
            }
            consts::RESP_CODE_SENT => {
                let exp_ack = u32::from_le_bytes([frame[1], frame[2], frame[3], frame[4]]);
                info!("Message sent.  Ack expected: {exp_ack:02x?}");
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
            consts::PUSH_CODE_ADVERT => {
                info!("Received new advert, requesting contact sync.");
                let get_contacts = GetContacts {
                    code: CMD_GET_CONTACTS,
                    since: Some(state.read().await.newest_advert_time),
                };
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdGetContacts(get_contacts),
                )
                .await;
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
            consts::PUSH_CODE_MSG_WAITING => {
                debug!("Received Message Waiting Indicator");
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdSyncNextMessage,
                )
                .await;
            }
            consts::RESP_CODE_CONTACT_MSG_RECV => {
                let contact_msg = ContactMsg::from_frame(&frame);
                debug!("Received contact message: {contact_msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ContactMsg(contact_msg));
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdSyncNextMessage,
                )
                .await;
            }
            consts::RESP_CODE_CONTACT_MSG_RECV_V3 => {
                let contact_msg = ContactMsgV3::from_frame(&frame);
                debug!("Received channel message: {contact_msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ContactMsgV3(contact_msg));
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdSyncNextMessage,
                )
                .await;
            }
            consts::RESP_CODE_CHANNEL_MSG_RECV => {
                let msg = ChannelMsg::from_frame(&frame);
                debug!("Received channel message: {msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ChannelMsg(msg));
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdSyncNextMessage,
                )
                .await;
            }
            consts::RESP_CODE_CHANNEL_MSG_RECV_V3 => {
                let msg = ChannelMsgV3::from_frame(&frame);
                debug!("Received channel message: {msg:?}");
                state
                    .write()
                    .await
                    .pending_messages
                    .push(MessageTypes::ChannelMsgV3(msg));
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdSyncNextMessage,
                )
                .await;
            }
            consts::RESP_CODE_NO_MORE_MESSAGES => {
                debug!("No more messages to sync.");
            }
            consts::PUSH_CODE_PATH_UPDATED => {
                info!("Received updated path for a contact, requesting full contact sync.");
                let get_contacts = GetContacts {
                    code: CMD_GET_CONTACTS,
                    since: Some(state.read().await.newest_advert_time),
                };
                let _ = send_command(
                    &state.read().await.to_radio_tx,
                    Commands::CmdGetContacts(get_contacts),
                )
                .await;
            }
            consts::PUSH_CODE_LOG_RX_DATA => {
                let snr = i32::from_le_bytes([frame[1], frame[2], frame[3], frame[4]]);
                let rssi = frame[5];
                let data = frame[6..].to_vec();
                debug!("Received log rx data: snr: {snr}, rssi: {rssi}, data: {data:?}");
            }
            _ => {
                warn!(
                    "unimplemented response code: {:02x} {:02x?}",
                    frame[0], frame
                );
            }
        }
    }
    Ok(())
}
