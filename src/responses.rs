use std::fmt;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use crate::{consts, AppError, Commands, CompanionState, HexData, InferredAdvert, MessageTypes};
use crate::commands::{send_command, GetContacts, MessageEnvelope, SendingMessageTypes};
use crate::commands::SendingMessageTypes::TxtMsg;
use crate::consts::CMD_GET_CONTACTS;
use crate::contact_mgmt::{Contact, PublicKey};

#[derive(Debug)]
pub enum Responses {
    SelfInfo(SelfInfo),
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
    Stats,
}
#[derive(Debug, Clone)]
pub struct SelfInfo {
    code: u8,
    r#type: u8,
    tx_power_dbm: u8,
    max_tx_power: u8,
    pub(crate) public_key: PublicKey,
    adv_lat: i32,
    adv_lon: i32,
    multi_acks: u8,
    advert_loc_policy: u8,
    telemetry_modes: u8,
    manual_add_contacts: u8,
    radio_freq: u32,
    radio_bw: u32,
    radio_sf: u8,
    radio_cr: u8,
    name: String,
}
impl SelfInfo {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut r#type = [0u8; 1];
        cursor.read_exact(&mut r#type).unwrap();
        let mut tx_power_dbm = [0u8; 1];
        cursor.read_exact(&mut tx_power_dbm).unwrap();
        let mut max_tx_power = [0u8; 1];
        cursor.read_exact(&mut max_tx_power).unwrap();
        let mut public_key = [0u8; 32];
        cursor.read_exact(&mut public_key).unwrap();
        let mut adv_lat = [0u8; 4];
        cursor.read_exact(&mut adv_lat).unwrap();
        let mut adv_lon = [0u8; 4];
        cursor.read_exact(&mut adv_lon).unwrap();
        let mut multi_acks = [0u8; 1];
        cursor.read_exact(&mut multi_acks).unwrap();
        let mut advert_loc_policy = [0u8; 1];
        cursor.read_exact(&mut advert_loc_policy).unwrap();
        let mut telemetry_modes = [0u8; 1];
        cursor.read_exact(&mut telemetry_modes).unwrap();
        let mut manual_add_contacts = [0u8; 1];
        cursor.read_exact(&mut manual_add_contacts).unwrap();
        let mut radio_freq = [0u8; 4];
        cursor.read_exact(&mut radio_freq).unwrap();
        let mut radio_bw = [0u8; 4];
        cursor.read_exact(&mut radio_bw).unwrap();
        let mut radio_sf = [0u8; 1];
        cursor.read_exact(&mut radio_sf).unwrap();
        let mut radio_cr = [0u8; 1];
        cursor.read_exact(&mut radio_cr).unwrap();
        let mut name = vec![];
        cursor.read_to_end(&mut name).unwrap();

        Self {
            code: code[0],
            r#type: r#type[0],
            tx_power_dbm: tx_power_dbm[0],
            max_tx_power: max_tx_power[0],
            public_key: PublicKey{ bytes: public_key },
            adv_lat: i32::from_le_bytes(adv_lat),
            adv_lon: i32::from_le_bytes(adv_lon),
            multi_acks: multi_acks[0],
            advert_loc_policy: advert_loc_policy[0],
            telemetry_modes: telemetry_modes[0],
            manual_add_contacts: manual_add_contacts[0],
            radio_freq: u32::from_le_bytes(radio_freq),
            radio_bw: u32::from_le_bytes(radio_bw),
            radio_sf: radio_sf[0],
            radio_cr: radio_cr[0],
            name: String::from_utf8_lossy(name.as_slice()).to_string(),
        }
    }
}
#[derive(Debug)]
pub struct DeviceInfo {
    code: u8,
    firmware_version: u8,
    max_contacts_div_2: u8,
    max_channels: u8,
    ble_pin: u32,
    firmware_build_date: String,
    manufacturer_model: String,
    semantic_version: String,
}

impl DeviceInfo {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut firmware_version = [0u8; 1];
        cursor.read_exact(&mut firmware_version).unwrap();
        let mut max_contacts_div_2 = [0u8; 1];
        cursor.read_exact(&mut max_contacts_div_2).unwrap();
        let mut max_channels = [0u8; 1];
        cursor.read_exact(&mut max_channels).unwrap();
        let mut ble_pin = [0u8; 4];
        cursor.read_exact(&mut ble_pin).unwrap();
        let mut firmware_build_date = [0u8; 12];
        cursor.read_exact(&mut firmware_build_date).unwrap();
        let mut manufacturer_model = [0u8; 40];
        cursor.read_exact(&mut manufacturer_model).unwrap();
        let mut semantic_version = [0u8; 20];
        cursor.read_exact(&mut semantic_version).unwrap();

        Self {
            code: code[0],
            firmware_version: firmware_version[0],
            max_contacts_div_2: max_contacts_div_2[0],
            max_channels: max_channels[0],
            ble_pin: u32::from_le_bytes(ble_pin),
            firmware_build_date: String::from_utf8_lossy(&firmware_build_date)
                .trim_end_matches('\0')
                .to_string(),
            manufacturer_model: String::from_utf8_lossy(&manufacturer_model)
                .trim_end_matches('\0')
                .to_string(),
            semantic_version: String::from_utf8_lossy(&semantic_version)
                .trim_end_matches('\0')
                .to_string(),
        }
    }
}
#[derive(Clone)]
pub struct PubkeyPrefix([u8;6]);
impl fmt::Display for PubkeyPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
impl fmt::Debug for PubkeyPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct ContactMsg {
    code: u8,
    pub pubkey_prefix: PubkeyPrefix,
    path_len: u8,
    txt_type: u8,
    sender_timestamp: u32,
    pub text: String,
}
impl ContactMsg {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut pubkey_prefix = [0u8; 6];
        cursor.read_exact(&mut pubkey_prefix).unwrap();
        let mut path_len = [0u8; 1];
        cursor.read_exact(&mut path_len).unwrap();
        let mut txt_type = [0u8; 1];
        cursor.read_exact(&mut txt_type).unwrap();
        let mut sender_timestamp = [0u8; 4];
        cursor.read_exact(&mut sender_timestamp).unwrap();
        let mut text = vec![];
        cursor.read_to_end(&mut text).unwrap();


        Self {
            code: code[0],
            pubkey_prefix: pubkey_prefix.into(),
            path_len: path_len[0],
            txt_type: txt_type[0],
            sender_timestamp: u32::from_le_bytes(sender_timestamp),
            text: String::from_utf8_lossy(&text)
                .trim_end_matches('\0')
                .to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContactMsgV3 {
    code: u8,
    snr: u8,
    reserved: [u8;2],
    pub pubkey_prefix: PubkeyPrefix,
    path_len: u8,
    txt_type: u8,
    sender_timestamp: u32,
    pub text: String,
}
impl ContactMsgV3 {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut snr = [0u8; 1];
        cursor.read_exact(&mut snr).unwrap();
        let mut reserved = [0u8;2];
        cursor.read_exact(&mut reserved).unwrap();
        let mut pubkey_prefix = [0u8; 6];
        cursor.read_exact(&mut pubkey_prefix).unwrap();
        let mut path_len = [0u8; 1];
        cursor.read_exact(&mut path_len).unwrap();
        let mut txt_type = [0u8; 1];
        cursor.read_exact(&mut txt_type).unwrap();
        let mut sender_timestamp = [0u8; 4];
        cursor.read_exact(&mut sender_timestamp).unwrap();
        let mut text = vec![];
        cursor.read_to_end(&mut text).unwrap();

        Self {
            code: code[0],
            snr: snr[0],
            reserved,
            pubkey_prefix: pubkey_prefix.into(),
            path_len: path_len[0],
            txt_type: txt_type[0],
            sender_timestamp: u32::from_le_bytes(sender_timestamp),
            text: String::from_utf8_lossy(text.as_slice())
                .trim_end_matches('\0')
                .to_string()
        }
    }
}
#[derive(Debug, Clone)]
pub struct ChannelMsg {
    code: u8,
    pub channel_id: u8,
    path_len: u8,
    txt_type: u8,
    sender_timestamp: u32,
    pub text: String,
}
impl ChannelMsg {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut channel_id = [0u8; 1];
        cursor.read_exact(&mut channel_id).unwrap();
        let mut path_len = [0u8; 1];
        cursor.read_exact(&mut path_len).unwrap();
        let mut txt_type = [0u8; 1];
        cursor.read_exact(&mut txt_type).unwrap();
        let mut sender_timestamp = [0u8; 4];
        cursor.read_exact(&mut sender_timestamp).unwrap();
        let mut text = vec![];
        cursor.read_to_end(&mut text).unwrap();


        Self {
            code: code[0],
            channel_id: channel_id[0],
            path_len: path_len[0],
            txt_type: txt_type[0],
            sender_timestamp: u32::from_le_bytes(sender_timestamp),
            text: String::from_utf8_lossy(text.as_slice())
                .trim_end_matches('\0')
                .to_string()
        }
    }
}
#[derive(Debug, Clone)]
pub struct ChannelMsgV3 {
    code: u8,
    snr: u8,
    reserved: [u8;2],
    pub channel_id: u8,
    path_len: u8,
    txt_type: u8,
    sender_timestamp: u32,
    pub text: String,
}
impl ChannelMsgV3 {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut snr = [0u8; 1];
        cursor.read_exact(&mut snr).unwrap();
        let mut reserved = [0u8;2];
        cursor.read_exact(&mut reserved).unwrap();
        let mut channel_id = [0u8; 1];
        cursor.read_exact(&mut channel_id).unwrap();
        let mut path_len = [0u8; 1];
        cursor.read_exact(&mut path_len).unwrap();
        let mut txt_type = [0u8; 1];
        cursor.read_exact(&mut txt_type).unwrap();
        let mut sender_timestamp = [0u8; 4];
        cursor.read_exact(&mut sender_timestamp).unwrap();
        let mut text = vec![];
        cursor.read_to_end(&mut text).unwrap();


        Self {
            code: code[0],
            snr: snr[0],
            reserved,
            channel_id: channel_id[0],
            path_len: path_len[0],
            txt_type: txt_type[0],
            sender_timestamp: u32::from_le_bytes(sender_timestamp),
            text: String::from_utf8_lossy(text.as_slice())
                .trim_end_matches('\0')
                .to_string()
        }
    }
}
#[derive(Clone, Debug)]
pub struct Confirmation {
    code: u8,
    pub(crate) ack_code: AckCode,
    round_trip: u32
}
impl Confirmation {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut ack_code = [0u8; 4];
        cursor.read_exact(&mut ack_code).unwrap();
        let mut round_trip = [0u8; 4];
        cursor.read_exact(&mut round_trip).unwrap();
        Self {
            code: code[0],
            ack_code: AckCode(ack_code),
            round_trip: u32::from_le_bytes(round_trip)
        }
    }
}
#[derive(Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct AckCode(pub [u8; 4]);

impl From<[u8;4]> for AckCode {
    fn from(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }
}
impl fmt::Display for AckCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
impl fmt::Debug for AckCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct LoginFailure {
    code: u8,
    reserved: u8,
    pub(crate) pub_key_prefix: [u8; 6],
}
impl LoginFailure {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut reserved = [0u8; 1];
        cursor.read_exact(&mut reserved).unwrap();
        let mut pub_key_prefix = [0u8; 6];
        cursor.read_exact(&mut pub_key_prefix).unwrap();
        Self {
            code: code[0],
            reserved: reserved[0],
            pub_key_prefix
        }
    }
}



#[derive(Clone, Debug)]
pub struct LoginSuccess {
    code: u8,
    permissions: u8,
    pub(crate) pub_key_prefix: [u8; 6],
    tag: i32,
    new_permissions: u8,
}
impl LoginSuccess {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut permissions = [0u8; 1];
        cursor.read_exact(&mut permissions).unwrap();
        let mut pub_key_prefix = [0u8; 6];
        cursor.read_exact(&mut pub_key_prefix).unwrap();
        let mut tag = [0u8; 4];
        cursor.read_exact(&mut tag).unwrap();
        let mut new_permissions = [0u8; 1];
        cursor.read_exact(&mut new_permissions).unwrap();
        Self {
            code: code[0],
            permissions: permissions[0],
            pub_key_prefix,
            tag: i32::from_le_bytes(tag),
            new_permissions: new_permissions[0]
        }
    }
}

#[derive(Debug,Clone)]
pub struct BattAndStorage {
    code: u8,
    pub(crate) milli_volts: u16,
    pub(crate) used_kb: u32,
    pub(crate) total_kb: u32
}
impl BattAndStorage {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut milli_volts = [0u8; 2];
        cursor.read_exact(&mut milli_volts).unwrap();
        let mut used_kb = [0u8; 4];
        cursor.read_exact(&mut used_kb).unwrap();
        let mut total_kb = [0u8; 4];
        cursor.read_exact(&mut total_kb).unwrap();
        Self {
            code: code[0],
            milli_volts: u16::from_le_bytes(milli_volts),
            used_kb: u32::from_le_bytes(used_kb),
            total_kb: u32::from_le_bytes(total_kb)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TuningParameters {
    pub(crate) code: u8,
    pub rxdelay_base: u32,
    pub airtime_factor: u32,
    pub reserved: [u8; 8]
}
impl TuningParameters {
    pub fn new(rxdelay_base: u32, airtime_factor: u32) -> Self {
    Self {
            code: consts::CMD_SET_TUNING_PARAMS,
            rxdelay_base,
            airtime_factor,
            reserved: [0u8; 8]
        }
    }
    pub fn to_frame(&self) -> Vec<u8> {
        let mut frame = vec![self.code];
        frame.extend_from_slice(&self.rxdelay_base.to_le_bytes());
        frame.extend_from_slice(&self.airtime_factor.to_le_bytes());
        frame.extend_from_slice(&self.reserved);
        frame
    }
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);
        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();
        let mut rxdelay_base = [0u8; 4];
        cursor.read_exact(&mut rxdelay_base).unwrap();
        let mut airtime_factor = [0u8; 4];
        cursor.read_exact(&mut airtime_factor).unwrap();
        let mut reserved = [0u8; 8];
        cursor.read_exact(&mut reserved).unwrap();
        Self {
            code: code[0],
            rxdelay_base: u32::from_le_bytes(rxdelay_base),
            airtime_factor: u32::from_le_bytes(airtime_factor),
            reserved
        }
    }
}



#[instrument(skip(state))]
pub(crate) async fn check_internal(state: Arc<RwLock<CompanionState>>) -> Result<(), AppError> {


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
            consts::RESP_CODE_TUNING_PARAMS => {
                let params = TuningParameters::from_frame(&frame);
                info!("Received new tuning parameters: {params:?}");
                state.write().await.tuning_parameters = Some(params.clone());
            }
            consts::PUSH_CODE_LOGIN_FAIL => {
                let mut lock = state.write().await;
                let login_failure = LoginFailure::from_frame(&frame);
                let pubkey = login_failure.pub_key_prefix.to_vec();
                error!("Login failed to {pubkey:?}");
                if let Some(c) = lock.contacts.iter_mut().find(|contact| contact.public_key.prefix() == login_failure.pub_key_prefix) {
                    c.logged_in = Some(false)
                }
            }
            consts::PUSH_CODE_LOGIN_SUCCESS => {
                let mut lock = state.write().await;
                let login_success = LoginSuccess::from_frame(&frame);
                let pubkey = login_success.pub_key_prefix.to_vec();
                info!("Login successful to {pubkey:?}");
                if let Some(c) = lock.contacts.iter_mut().find(|contact| contact.public_key.prefix() == login_success.pub_key_prefix) {
                    c.logged_in = Some(true)
                }
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
                info!("Received Message Waiting Indicator");
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
impl From<[u8;6]> for PubkeyPrefix {
    fn from(bytes: [u8;6]) -> Self {
        Self(bytes)
    }
}