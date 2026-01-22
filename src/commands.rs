use serde::{Deserialize, Serialize};
use crate::consts::CMD_SET_RADIO_PARAMS;
use crate::contact_mgmt::PublicKey;

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
    CmdSetRadioTxPower,
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