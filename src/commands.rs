
use serde::{Serialize, Deserialize};

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
    CmdSendTxtMsg,
    CmdSendChannelTxtMsg,
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
    CmdGetStats
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppStart {
    pub code: u8,
    pub app_ver: u8,
    pub reserved: [u8; 6],
    pub app_name: String
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeviceQuery {
    pub code: u8,
    pub app_target_ver: u8
}

pub struct GetContacts {
    pub code: u8,
    pub since: Option<u32>
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Reboot {
    pub(crate) code: u8,
    pub(crate) text: String
}