use std::fmt;
use std::io::{Cursor, Read};

#[derive(Clone, Copy)]
pub struct PublicKey {
    pub(crate) bytes: [u8; 32],
}

impl PublicKey {
    pub fn prefix(&self) -> Vec<u8> {
        self.bytes[0..6].to_vec()
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub public_key: PublicKey,
    pub adv_type: u8,
    pub flags: u8,
    pub out_path_len: i8,
    pub out_path: [u8; 64],
    pub adv_name: String,
    pub last_advert: u32,
    pub adv_lat: i32,
    pub adv_lon: i32,
    pub lastmod: u32,
}
impl Contact {
    pub fn from_frame(frame: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(frame);

        let mut code = [0u8; 1];
        cursor.read_exact(&mut code).unwrap();

        let mut public_key = [0u8; 32];
        cursor.read_exact(&mut public_key).unwrap();

        let mut adv_type = [0u8; 1];
        cursor.read_exact(&mut adv_type).unwrap();

        let mut flags = [0u8; 1];
        cursor.read_exact(&mut flags).unwrap();
        let mut out_path_len = [0u8; 1];
        cursor.read_exact(&mut out_path_len).unwrap();
        let mut out_path = [0u8; 64];
        cursor.read_exact(&mut out_path).unwrap();
        let mut adv_name = [0u8; 32];
        cursor.read_exact(&mut adv_name).unwrap();
        let mut last_advert = [0u8; 4];
        cursor.read_exact(&mut last_advert).unwrap();
        let mut adv_lat = [0u8; 4];
        cursor.read_exact(&mut adv_lat).unwrap();
        let mut adv_lon = [0u8; 4];
        cursor.read_exact(&mut adv_lon).unwrap();
        let mut lastmod = [0u8; 4];
        cursor.read_exact(&mut lastmod).unwrap();

        Self {
            public_key: PublicKey { bytes: public_key },
            adv_type: adv_type[0],
            flags: flags[0],
            out_path_len: out_path_len[0] as i8,
            out_path,
            adv_name: String::from_utf8(adv_name.to_vec())
                .unwrap()
                .trim_end_matches('\0')
                .to_string(),
            last_advert: u32::from_le_bytes(last_advert),
            adv_lat: i32::from_le_bytes(adv_lat),
            adv_lon: i32::from_le_bytes(adv_lon),
            lastmod: u32::from_le_bytes(lastmod),
        }
    }
}
