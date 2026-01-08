use std::io::ErrorKind;
use std::process::exit;
use tokio::sync::mpsc;
use tokio::time::Duration;
use crate::AppError;
use crate::consts::{SERIAL_INBOUND, SERIAL_LOOP_SLEEP_MS, SERIAL_OUTBOUND, TIMEOUT_SERIAL_MS};
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct SerialFrame {
    pub(crate) delimiter: u8,
    #[serde(with = "fixed_length_u16")]
    pub(crate) frame_length: u16,
    #[serde(with = "raw_vec")]
    pub(crate) frame: Vec<u8>
}

pub async fn serial_loop(
    port: String,
    to_radio: &mut mpsc::Receiver<SerialFrame>,
    from_radio: &mpsc::Sender<SerialFrame>) {

    let mut fd = serialport::new(&port, 115200)
        .timeout(Duration::from_millis(TIMEOUT_SERIAL_MS))
        .open()
        .unwrap_or_else(|e| { error!("Failed to open serial port: {}", e); exit(1); } );

    let mut buffer = [0; 1024];
    let mut accumulator = Vec::new();
    loop {
        // transmit outgoing messages
        if let Ok(msg) = to_radio.try_recv() {
            // Instead of postcard-serializing the whole struct:
            // 1. Delimiter (1 byte)
            // 2. Length (2 bytes, LE)
            // 3. Raw Frame (N bytes)
            let mut data = Vec::with_capacity(3 + msg.frame.len());
            data.push(msg.delimiter);
            data.extend_from_slice(&msg.frame_length.to_le_bytes());
            data.extend_from_slice(&msg.frame);

            info!("Sending serial frame: {:02x?}", data);
            fd.write_all(&data).unwrap_or_else(|e| { 
                error!("Failed to write serial port: {}", e); 
                exit(1); 
            });
        }

        // check for incoming messages
        match fd.read(&mut buffer) {
            Ok(d) if d > 0 => {
                accumulator.extend_from_slice(&buffer[..d]);
                let formatted = accumulator.clone().iter()
                    .map(|b| format!("0x{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(",");
                info!("accumulator: {formatted}");
                match decode_frame(&accumulator) {
                    Ok(frame) => {
                        accumulator.clear();
                        from_radio.send(frame).await.unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
                    },
                    Err(e) if e.kind() == DecodeErrorKind::FrameTooShort => {},
                    Err(e) if e.kind() == DecodeErrorKind::FrameTooLong => accumulator.clear(),
                    Err(e) => println!("Failed to decode frame: {}", e)
                }
            },
            Ok(_) => (),
            Err(ref e) if e.kind() == ErrorKind::TimedOut => (),
            Err(e) => {
                println!("Serial read error: {e}");
            }
        }
        tokio::time::sleep(Duration::from_millis(SERIAL_LOOP_SLEEP_MS)).await;
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    InvalidDelimiter,
    FrameTooShort,
    FrameTooLong,
    Misc
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DecodeError {
    #[error("Misc: {0}")]
    Misc(String),
    #[error("Frame too short")]
    FrameTooShort,
    #[error("Frame too long")]
    FrameTooLong,
    #[error("Invalid delimiter")]
    InvalidDelimiter,

}

impl DecodeError {
    pub(crate) fn kind(&self) -> DecodeErrorKind {
        match self {
            DecodeError::InvalidDelimiter => DecodeErrorKind::InvalidDelimiter,
            DecodeError::FrameTooShort => DecodeErrorKind::FrameTooShort,
            DecodeError::FrameTooLong => DecodeErrorKind::FrameTooLong,
            DecodeError::Misc(_) => DecodeErrorKind::Misc
        }
    }
}

pub fn decode_frame(in_frame: &[u8]) -> Result<SerialFrame,DecodeError> {
    // if in_frame.len() == 1 && in_frame[0] == 62 {
    //     return Ok(SerialFrame {
    //         delimiter: in_frame[0],
    //         frame_length: 0,
    //         frame: vec![]
    //     });
    // }
    if in_frame.len() < 4 {
        return Err(DecodeError::Misc(format!("Frame too short: {}", in_frame.len())))
    }
    let frame_length = u16::from_le_bytes([in_frame[1], in_frame[2]]);
    if in_frame.len() > frame_length as usize + 3{
        return Err(DecodeError::FrameTooLong)
    }
    if in_frame.len() < frame_length as usize + 3 {
        return Err(DecodeError::FrameTooShort)
    }
    if in_frame[0] != SERIAL_INBOUND && in_frame[0] != SERIAL_OUTBOUND {
        return Err(DecodeError::InvalidDelimiter)
    }
    let delimiter = in_frame[0];
    let frame_length = u16::from_le_bytes([in_frame[1], in_frame[2]]);
    let frame = in_frame[3..].to_vec();
    Ok(SerialFrame { delimiter, frame_length, frame})
}

mod fixed_length_u16 {
    use serde::{Deserialize, Deserializer, Serializer, Serialize};

    pub fn serialize<S>(value: &u16, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = value.to_le_bytes(); // or to_be_bytes() depending on protocol
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u16, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 2] = Deserialize::deserialize(deserializer)?;
        Ok(u16::from_le_bytes(bytes)) // or from_be_bytes()
    }
}
mod raw_vec {
    use serde::{Deserialize, Deserializer, Serializer, Serialize};
    use serde::ser::SerializeSeq;

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(data.len()))?;
        for byte in data {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = Deserialize::deserialize(deserializer)?;
        Ok(bytes.to_vec())
    }
}