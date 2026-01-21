use serde::{Deserialize, Serialize};
use crate::contact_mgmt::PublicKey;

#[derive(Debug, Clone)]
pub enum Commands {
    CmdDeviceQuery(DeviceQuery),
    CmdAppStart(AppStart),
    CmdGetContacts(GetContacts),
    CmdGetDeviceTime,
    CmdSetDeviceTime,
    CmdSendSelfAdvert(AdvertisementMode),
    CmdSetAdvertName,
    CmdSetAdvertLatLon,
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
    CmdSetRadioParams,
    CmdSetRadioTxPower,
    CmdResetPath,
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