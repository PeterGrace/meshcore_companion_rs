//region Command Codes (consts)
const CMD_DEVICE_QEURY: u8 = 22;
const CMD_APP_START: u8 = 1;
const CMD_GET_CONTACTS :u8 = 4;
const CMD_GET_DEVICE_TIME: u8 = 5;
const CMD_SET_DEVICE_TIME: u8 = 6;
const CMD_SEND_SELF_ADVERT: u8 = 7;
const CMD_SET_ADVERT_NAME: u8 = 8;
const CMD_SET_ADVERT_LATLON: u8 = 14;
const CMD_SYNC_NEXT_MESSAGE: u8 = 10;
const CMD_ADD_UPDATE_CONTACT: u8 = 9;
const CMD_REMOVE_CONTACT: u8 = 15;
const CMD_SHARE_CONTACT: u8 = 16;
const CMD_EXPORT_CONTACT: u8 = 17;
const CMD_IMPORT_CONTACT: u8 = 18;
const CMD_REBOOT: u8 = 19;
const CMD_GET_BATT_AND_STORAGE: u8 = 20;
const CMD_SET_TUNING_PARAMS: u8 = 21;
const CMD_SET_OTHER_PARAMS: u8 = 38;
const CMD_SEND_TXT_MSG: u8 = 2;
const CMD_SEND_CHANNEL_TXT_MSG: u8 = 3;
const CMD_SET_RADIO_PARAMS: u8 = 11;
const CMD_SET_RADIO_TX_POWER: u8 = 12;
const CMD_RESET_PATH: u8 = 13;
const CMD_SEND_RAW_DATA: u8 = 25;
const CMD_SEND_LOGIN: u8 = 26;
const CMD_SEND_STATUS_REQ: u8 = 27;
const CMD_SEND_TRACE_PATH: u8 = 36;
const CMD_SEND_TELEMETRY_REQ: u8 = 39;
const CMD_GET_CUSTOM_VARS: u8 = 40;
const CMD_SET_CUSTOM_VARS: u8 = 41;
const CMD_GET_ADVERT_PATH: u8 = 42;
const CMD_GET_TUNING_PARAMS: u8 = 43;
const CMD_SEND_BINARY_REQ: u8 = 50;
const CMD_FACTORY_RESET: u8 = 51;
const CMD_SEND_CONTROL_DATA: u8 = 55;
const CMD_GET_STATS: u8 = 56;
//endregion

//region Response Codes (consts)
const RESP_CODE_DEVICE_INFO: u8 = 13;
const RESP_CODE_SELF_INFO: u8 = 5;
const RESP_CODE_CONTACTS_START: u8 = 2;
const RESP_CODE_CONTACT: u8 = 3;
const RESP_CODE_END_OF_CONTACTS: u8 = 4;
const RESP_CODE_CURR_TIME: u8 = 9;
const RESP_CODE_OK: u8 = 0;
const RESP_CODE_ERR: u8 = 1;

const RESP_CODE_NO_MORE_MESSAGES: u8 = 10;
const RESP_CODE_CONTACT_MSG_RECV: u8 = 7;
const RESP_CODE_CHANNEL_MSG_RECV: u8 = 8;
const RESP_CODE_CONTACT_MSG_RECV_V3: u8 = 16;
const RESP_CODE_CHANNEL_MSG_RECV_V3: u8 = 17;
const RESP_CODE_EXPORT_CONTACT: u8 = 11;
const RESP_CODE_BATT_AND_STORAGE: u8 = 12;
const RESP_CODE_SENT: u8 = 6;
const RESP_CODE_ADVERT_PATH: u8 = 22;
const RESP_CODE_STATUS: u8 = 24;

//endregion

//region Push Codes (consts)
const PUSH_CODE_ADVERT: u8 = 0x80;
const PUSH_CODE_PATH_UPDATED: u8 = 0x81;
const PUSH_CODE_SEND_CONFIRMED: u8 = 0x82;
const PUSH_CODE_MSG_WAITING: u8 = 0x83;
const PUSH_CODE_RAW_DATA: u8 = 0x84;
const PUSH_CODE_LOGIN_SUCCESS: u8 = 0x85;
const PUSH_CODE_LOGIN_FAIL: u8 = 0x86;
const PUSH_CODE_STATUS_RESPONSE: u8 = 0x87;
const PUSH_CODE_TRACE_DATA: u8 = 0x89;
const PUSH_CODE_NEW_ADVERT: u8 = 0x8A;
const PUSH_CODE_TELEMETRY_RESPONSE: u8 = 0x8B;
const PUSH_CODE_BINARY_RESPONSE: u8 = 0x8C;
const PUSH_CODE_CONTROL_DATA: u8 = 0x8D;
//endregion

//region error codes (consts)
const ERR_CODE_UNSUPPORTED_CMD: u8 = 1;
const ERR_CODE_NOT_FOUND: u8 = 2;
const ERR_CODE_TABLE_FULL: u8 = 3;
const ERR_CODE_BAD_STATE: u8 = 4;
const ERR_CODE_FILE_IO_ERROR: u8 = 5;
const ERR_CODE_ILLEGAL_ARG: u8 = 6;
//endregion

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
pub enum Responses {
    ContactsStart,
    Contact,
    EndOfContacts,
    CurrTime,
    Ok,
    Err,
    NoMoreMessages,
    ContactMsgRecv,
    ContactMsgRecvV3,
    ChannelMsgRecv,
    ChannelMsgRecvV3,
    ExportContact,
    BattAndStorage,
    Sent,
    AdvertPath,
    Stats
}

pub enum PushCodes {
    Advert,
    PathUpdated,
    SendConfirmed,
    MsgWaiting,
    RawData,
    LoginSuccess,
    LoginFail,
    StatusResponse,
    TraceData,
    NewAdvert,
    TelemetryResponse,
    BinaryResponse,
    ControlData
}

pub struct AppStart {
    code: u8,
    app_ver: u8,
    reserved: Vec<u8>,
    app_name: String
}
pub struct DeviceQuery {
    code: u8,
    app_target_ver: u8
}

pub struct GetContacts {
    code: u8,
    since: Option<u32>
}