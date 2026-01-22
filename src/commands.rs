use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::{consts, AppError, CompanionState};
use crate::consts::{CMD_EXPORT_CONTACT, CMD_GET_BATT_AND_STORAGE, CMD_GET_DEVICE_TIME, CMD_REMOVE_CONTACT, CMD_RESET_PATH, CMD_SEND_SELF_ADVERT, CMD_SET_ADVERT_LATLON, CMD_SET_ADVERT_NAME, CMD_SET_DEVICE_TIME, CMD_SET_RADIO_PARAMS};
use crate::contact_mgmt::PublicKey;
use crate::serial_actor::SerialFrame;

#[derive(Debug, Clone)]
pub enum Commands {
    CmdDeviceQuery(DeviceQuery),
    CmdAppStart(AppStart),
    CmdGetContacts(GetContacts),
    CmdGetDeviceTime,
    CmdSetDeviceTime,
    CmdSendSelfAdvert(AdvertisementMode),
    CmdSetAdvertName(String),
    CmdSetAdvertLatLon(LatLonAlt),
    CmdSyncNextMessage,
    CmdAddUpdateContact,
    CmdRemoveContact(PublicKey),
    CmdShareContact,
    CmdExportContact(Option<PublicKey>),
    CmdImportContact,
    CmdReboot,
    CmdGetBattAndStorage,
    CmdSetTuningParams,
    CmdSetOtherParams,
    CmdSendTxtMsg(SendTxtMsg),
    CmdSendChannelTxtMsg(SendChannelTxtMsg),
    CmdSetRadioParams(RadioParameters),
    CmdSetRadioTxPower(u8),
    CmdResetPath(PublicKey),
    CmdSendRawData,
    CmdSendLogin(LoginData),
    CmdSendStatusReq,
    CmdSendTracePath,
    CmdSendTelemetryReq,
    CmdGetCustomVars,
    CmdSetCustomVar,
    CmdGetAdvertPath,
    CmdGetTuningParams,
    CmdSendBinaryReq,
    CmdFactoryReset,
    CmdSendControlData,
    CmdGetStats,
}

#[derive(Debug, Clone)]
pub struct LoginData {
    pub code: u8,
    pub public_key: PublicKey,
    pub password: String
}
impl LoginData {
    pub fn to_frame(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.code);
        data.extend_from_slice(&self.public_key.bytes);
        data.extend_from_slice(self.password.as_bytes());
        data
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppStart {
    pub code: u8,
    pub app_ver: u8,
    pub reserved: [u8; 6],
    pub app_name: String,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DeviceQuery {
    pub code: u8,
    pub app_target_ver: u8,
}

#[derive(Debug, Clone)]
pub struct GetContacts {
    pub code: u8,
    pub since: Option<u32>,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Reboot {
    pub(crate) code: u8,
    pub(crate) text: String,
}

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    pub(crate) msg: SendingMessageTypes,
    pub(crate) last_attempt_timestamp: u128,
}

#[derive(Debug, Clone)]
pub enum SendingMessageTypes {
    TxtMsg(SendTxtMsg),
    ChannelMsg(SendChannelTxtMsg)
}

#[derive(Debug, Clone)]
pub struct SendTxtMsg {
    pub code: u8,
    pub txt_type: u8,
    pub attempt: u8,
    pub sender_timestamp: u32,
    pub pubkey_prefix: [u8; 6],
    pub text: String,
    pub timeout: Option<u32>,
}
impl SendTxtMsg {
    pub fn to_frame(&self) -> Vec<u8> {
        let mut frame = vec![self.code, self.txt_type, self.attempt];
        frame.extend_from_slice(self.sender_timestamp.to_le_bytes().as_slice());
        frame.extend_from_slice(&self.pubkey_prefix);
        frame.extend_from_slice(&self.text.as_bytes());
        frame
    }
}
#[derive(Clone, Debug)]
pub struct SendChannelTxtMsg {
    pub code: u8,
    pub txt_type: u8,
    pub channel_idx: u8,
    pub sender_timestamp: u32,
    pub text: String
}

impl SendChannelTxtMsg {
    pub(crate) fn to_frame(&self) -> Vec<u8> {
        let mut frame = vec![self.code, self.txt_type, self.channel_idx];
        frame.extend_from_slice(self.sender_timestamp.to_le_bytes().as_slice());
        frame.extend_from_slice(&self.text.as_bytes());
        frame   
    }
}
#[derive(Debug, Clone)]
pub enum AdvertisementMode {
    ZeroHop = 0,
    Flood = 1
}
#[derive(Debug, Clone)]
pub struct LatLonAlt {
    pub latitude: i32,
    pub longitude: i32,
    pub altitude: i32
}

impl LatLonAlt {
    pub(crate) fn to_frame(&self) -> Vec<u8> {
        let mut frame = vec![];
        frame.extend_from_slice(&self.latitude.to_le_bytes());
        frame.extend_from_slice(&self.longitude.to_le_bytes());
        frame.extend_from_slice(&self.altitude.to_le_bytes());
        frame  
    }
}

impl LatLonAlt {
    pub fn from_decimal(lat: f64, lon: f64, alt: f64) -> Self {
        Self {
            latitude: (lat * 1E6) as i32,
            longitude: (lon * 1E6) as i32,
            altitude: (alt * 1E6) as i32
        }
    }
    pub fn to_decimal(&self) -> (f64, f64, f64) {
        (self.latitude as f64 / 1E6, self.longitude as f64 / 1E6, self.altitude as f64 / 1E6)
    }
}
#[derive(Debug, Clone)]
pub struct RadioParameters {
    pub code: u8,
    pub radio_freq:u32,
    pub radio_bw: u32,
    pub radio_sf: u8,
    pub radio_cr: u8
}
impl RadioParameters {
    pub fn new(radio_freq: u32, radio_bw: u32, radio_sf: u8, radio_cr: u8) -> Self {
        Self {
            code: CMD_SET_RADIO_PARAMS,
            radio_freq,
            radio_bw,
            radio_sf,
            radio_cr
        }
    }
    pub fn to_frame(&self) -> Vec<u8> {
        let mut frame = vec![];
        frame.extend_from_slice(&self.code.to_le_bytes());
        frame.extend_from_slice(&self.radio_freq.to_le_bytes());
        frame.extend_from_slice(&self.radio_bw.to_le_bytes());
        frame.extend_from_slice(&self.radio_sf.to_le_bytes());
        frame.extend_from_slice(&self.radio_cr.to_le_bytes());
        frame
    }
}

pub async fn send_command(
    state: &Arc<RwLock<CompanionState>>,
    cmd: Commands,
) -> Result<(), AppError> {
    let tx = state.write().await.to_radio_tx.clone();
    match cmd {
        Commands::CmdSetRadioTxPower(power) => {
            let data = vec![CMD_SET_RADIO_PARAMS, power];
            let frame: SerialFrame = SerialFrame::from_data(data);
            tx.send(frame)
                .await
                .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
            state.write().await.command_queue.push_back(cmd);
            Ok(())
        }
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
