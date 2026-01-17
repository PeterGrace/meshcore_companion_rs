use serde::{Deserialize, Serialize};

pub enum Commands {
    CmdDeviceQuery(DeviceQuery),
    CmdAppStart(AppStart),
    CmdGetContacts(GetContacts),
    CmdGetDeviceTime,
    CmdSetDeviceTime,
    CmdSendSelfAdvert,
    CmdSetAdvertName,
    CmdSetAdvertLatLon,
    CmdSyncNextMessage,
    CmdAddUpdateContact,
    CmdRemoveContact,
    CmdShareContact,
    CmdExportContact,
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
    CmdSendLogin,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppStart {
    pub code: u8,
    pub app_ver: u8,
    pub reserved: [u8; 6],
    pub app_name: String,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeviceQuery {
    pub code: u8,
    pub app_target_ver: u8,
}

pub struct GetContacts {
    pub code: u8,
    pub since: Option<u32>,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Reboot {
    pub(crate) code: u8,
    pub(crate) text: String,
}

pub struct SendTxtMsg {
    pub code: u8,
    pub txt_type: u8,
    pub attempt: u8,
    pub sender_timestamp: u32,
    pub pubkey_prefix: [u8; 6],
    pub text: String
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