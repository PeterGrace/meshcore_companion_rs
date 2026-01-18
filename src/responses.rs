use std::fmt;
use std::io::{Cursor, Read};
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
#[derive(Debug)]
pub struct SelfInfo {
    code: u8,
    r#type: u8,
    tx_power_dbm: u8,
    max_tx_power: u8,
    public_key: [u8; 32],
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
            public_key,
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
            name: String::from_utf8(name).unwrap(),
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
            firmware_build_date: String::from_utf8(firmware_build_date.to_vec())
                .unwrap()
                .trim_end_matches('\0')
                .to_string(),
            manufacturer_model: String::from_utf8(manufacturer_model.to_vec())
                .unwrap()
                .trim_end_matches('\0')
                .to_string(),
            semantic_version: String::from_utf8(semantic_version.to_vec())
                .unwrap()
                .trim_end_matches('\0')
                .to_string(),
        }
    }
}
#[derive(Debug)]
pub struct ContactMsg {
    code: u8,
    pub pubkey_prefix: [u8; 6],
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
            pubkey_prefix,
            path_len: path_len[0],
            txt_type: txt_type[0],
            sender_timestamp: u32::from_le_bytes(sender_timestamp),
            text: String::from_utf8(text)
                .unwrap()
                .trim_end_matches('\0')
                .to_string()
        }
    }
}

#[derive(Debug)]
pub struct ContactMsgV3 {
    code: u8,
    snr: u8,
    reserved: [u8;2],
    pub pubkey_prefix: [u8; 6],
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
            pubkey_prefix,
            path_len: path_len[0],
            txt_type: txt_type[0],
            sender_timestamp: u32::from_le_bytes(sender_timestamp),
            text: String::from_utf8(text)
                .unwrap()
                .trim_end_matches('\0')
                .to_string()
        }
    }
}
#[derive(Debug)]
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
            text: String::from_utf8(text)
                .unwrap()
                .trim_end_matches('\0')
                .to_string()
        }
    }
}
#[derive(Debug)]
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
            text: String::from_utf8(text)
                .unwrap()
                .trim_end_matches('\0')
                .to_string()
        }
    }
}
#[derive(Clone, Debug)]
pub struct Confirmation {
    code: u8,
    ack_code: AckCode,
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