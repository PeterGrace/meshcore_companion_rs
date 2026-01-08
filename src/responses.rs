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
    Stats
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
    name: String
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
            name: String::from_utf8(name).unwrap()

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
    semantic_version: String
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
        let mut semantic_version = [0u8;20];
        cursor.read_exact(&mut semantic_version).unwrap();

        Self {
            code: code[0],
            firmware_version: firmware_version[0],
            max_contacts_div_2: max_contacts_div_2[0],
            max_channels: max_channels[0],
            ble_pin: u32::from_le_bytes(ble_pin),
            firmware_build_date: String::from_utf8(firmware_build_date.to_vec()).unwrap().trim_end_matches('\0').to_string(),
            manufacturer_model: String::from_utf8(manufacturer_model.to_vec()).unwrap().trim_end_matches('\0').to_string(),
            semantic_version: String::from_utf8(semantic_version.to_vec()).unwrap().trim_end_matches('\0').to_string()
        }
        }
    }
