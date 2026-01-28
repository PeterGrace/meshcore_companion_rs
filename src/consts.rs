use crate::commands::RadioParameters;

//region Command Codes (consts)
pub const CMD_DEVICE_QEURY: u8 = 22;
pub const CMD_APP_START: u8 = 1;
pub const CMD_GET_CONTACTS: u8 = 4;
pub const CMD_GET_DEVICE_TIME: u8 = 5;
pub const CMD_SET_DEVICE_TIME: u8 = 6;
pub const CMD_SEND_SELF_ADVERT: u8 = 7;
pub const CMD_SET_ADVERT_NAME: u8 = 8;
pub const CMD_SET_ADVERT_LATLON: u8 = 14;
pub const CMD_SYNC_NEXT_MESSAGE: u8 = 10;
pub const CMD_ADD_UPDATE_CONTACT: u8 = 9;
pub const CMD_REMOVE_CONTACT: u8 = 15;
pub const CMD_SHARE_CONTACT: u8 = 16;
pub const CMD_EXPORT_CONTACT: u8 = 17;
pub const CMD_IMPORT_CONTACT: u8 = 18;
pub const CMD_REBOOT: u8 = 19;
pub const CMD_GET_BATT_AND_STORAGE: u8 = 20;
pub const CMD_SET_TUNING_PARAMS: u8 = 21;
pub const CMD_SET_OTHER_PARAMS: u8 = 38;
pub const CMD_SEND_TXT_MSG: u8 = 2;
pub const CMD_SEND_CHANNEL_TXT_MSG: u8 = 3;
pub const CMD_SET_RADIO_PARAMS: u8 = 11;
pub const CMD_SET_RADIO_TX_POWER: u8 = 12;
pub const CMD_RESET_PATH: u8 = 13;
pub const CMD_SEND_RAW_DATA: u8 = 25;
pub const CMD_SEND_LOGIN: u8 = 26;
pub const CMD_SEND_STATUS_REQ: u8 = 27;
pub const CMD_LOGOUT: u8 = 29;
pub const CMD_SEND_TRACE_PATH: u8 = 36;
pub const CMD_SEND_TELEMETRY_REQ: u8 = 39;
pub const CMD_GET_CUSTOM_VARS: u8 = 40;
pub const CMD_SET_CUSTOM_VARS: u8 = 41;
pub const CMD_GET_ADVERT_PATH: u8 = 42;
pub const CMD_GET_TUNING_PARAMS: u8 = 43;
pub const CMD_SEND_BINARY_REQ: u8 = 50;
pub const CMD_FACTORY_RESET: u8 = 51;
pub const CMD_SEND_CONTROL_DATA: u8 = 55;
pub const CMD_GET_STATS: u8 = 56;

//endregion

//region Response Codes (consts)
pub const RESP_CODE_DEVICE_INFO: u8 = 13;
pub const RESP_CODE_SELF_INFO: u8 = 5;
pub const RESP_CODE_CONTACTS_START: u8 = 2;
pub const RESP_CODE_CONTACT: u8 = 3;
pub const RESP_CODE_END_OF_CONTACTS: u8 = 4;
pub const RESP_CODE_CURR_TIME: u8 = 9;
pub const RESP_CODE_OK: u8 = 0;
pub const RESP_CODE_ERR: u8 = 1;

pub const RESP_CODE_NO_MORE_MESSAGES: u8 = 10;
pub const RESP_CODE_CONTACT_MSG_RECV: u8 = 7;
pub const RESP_CODE_CHANNEL_MSG_RECV: u8 = 8;
pub const RESP_CODE_CONTACT_MSG_RECV_V3: u8 = 16;
pub const RESP_CODE_CHANNEL_MSG_RECV_V3: u8 = 17;
pub const RESP_CODE_EXPORT_CONTACT: u8 = 11;
pub const RESP_CODE_BATT_AND_STORAGE: u8 = 12;
pub const RESP_CODE_SENT: u8 = 6;
pub const RESP_CODE_ADVERT_PATH: u8 = 22;
pub const RESP_CODE_TUNING_PARAMS: u8 = 23;
pub const RESP_CODE_STATUS: u8 = 24;
//endregion

//region Push Codes (consts)
pub const PUSH_CODE_ADVERT: u8 = 0x80;
pub const PUSH_CODE_PATH_UPDATED: u8 = 0x81;
pub const PUSH_CODE_SEND_CONFIRMED: u8 = 0x82;
pub const PUSH_CODE_MSG_WAITING: u8 = 0x83;
pub const PUSH_CODE_RAW_DATA: u8 = 0x84;
pub const PUSH_CODE_LOGIN_SUCCESS: u8 = 0x85;
pub const PUSH_CODE_LOGIN_FAIL: u8 = 0x86;
pub const PUSH_CODE_STATUS_RESPONSE: u8 = 0x87;
pub const PUSH_CODE_LOG_RX_DATA: u8 = 0x88;
pub const PUSH_CODE_TRACE_DATA: u8 = 0x89;
pub const PUSH_CODE_NEW_ADVERT: u8 = 0x8A;
pub const PUSH_CODE_TELEMETRY_RESPONSE: u8 = 0x8B;
pub const PUSH_CODE_BINARY_RESPONSE: u8 = 0x8C;
pub const PUSH_CODE_CONTROL_DATA: u8 = 0x8D;
//endregion

//region error codes (consts)
pub const ERR_CODE_UNSUPPORTED_CMD: u8 = 1;
pub const ERR_CODE_NOT_FOUND: u8 = 2;
pub const ERR_CODE_TABLE_FULL: u8 = 3;
pub const ERR_CODE_BAD_STATE: u8 = 4;
pub const ERR_CODE_FILE_IO_ERROR: u8 = 5;
pub const ERR_CODE_ILLEGAL_ARG: u8 = 6;
//endregion

pub const MPSC_BUFFER_DEPTH: usize = 100;
pub const SERIAL_LOOP_SLEEP_MS: u64 = 10;
pub const TIMEOUT_SERIAL_MS: u64 = 100;

// hex: 0x3e
pub const SERIAL_INBOUND: u8 = 62;
// hex: 0x3c
pub const SERIAL_OUTBOUND: u8 = 60;

pub const USA_RADIO_PRESET: RadioParameters = RadioParameters { code: 11, radio_freq: 910525, radio_bw: 62500, radio_sf: 7, radio_cr: 5 };