use crate::AppError;
use crate::consts::{SERIAL_INBOUND, SERIAL_LOOP_SLEEP_MS, SERIAL_OUTBOUND, TIMEOUT_SERIAL_MS};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::process::exit;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{debug, error, info, trace};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SerialFrame {
    pub(crate) delimiter: u8,
    pub(crate) frame_length: u16,
    pub(crate) frame: Vec<u8>,
}

    pub async fn serial_loop(
        port: String,
        to_radio: &mut mpsc::Receiver<SerialFrame>,
        from_radio: &mpsc::Sender<SerialFrame>,
    ) {
        // Use a loop here to allow for reconnection if the serial port drops
        loop {
            let mut fd = match serialport::new(&port, 115200)
                .timeout(Duration::from_millis(TIMEOUT_SERIAL_MS))
                .open() {
                    Ok(port) => port,
                    Err(e) => {
                        error!("Failed to open serial port {}: {}. Retrying in 5s...", port, e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

            info!("Serial port {} opened successfully.", port);

            let mut buffer = [0; 1024];
            let mut accumulator = Vec::new();
            
            // Inner loop for the actual communication
            loop {
                // transmit outgoing messages
                // Use while let instead of if let to drain the queue
                while let Ok(msg) = to_radio.try_recv() {
                    let mut data = Vec::with_capacity(3 + msg.frame.len());
                    data.push(msg.delimiter);
                    data.extend_from_slice(&msg.frame_length.to_le_bytes());
                    data.extend_from_slice(&msg.frame);

                    debug!("Sending serial frame: {:02x?}", data);
                    if let Err(e) = fd.write_all(&data) {
                        error!("Failed to write to serial port: {}. Restarting connection...", e);
                        break; // Break inner loop to trigger reconnect
                    }
                }

                // check for incoming messages
                match fd.read(&mut buffer) {
                    Ok(d) if d > 0 => {
                        accumulator.extend_from_slice(&buffer[..d]);

                        loop {
                            let formatted = accumulator
                                .clone()
                                .iter()
                                .map(|b| format!("0x{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(",");
                            debug!("accumulator: {formatted}");

                            match decode_frame(&accumulator) {
                                Ok((frame, None)) => {
                                    from_radio
                                        .send(frame)
                                        .await
                                        .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
                                    accumulator.clear();
                                    break;
                                }
                                Ok((frame, Some(residual))) => {
                                    trace!("Residual frame: {:02x?}", residual);
                                    from_radio
                                        .send(frame)
                                        .await
                                        .unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
                                    accumulator = residual;
                                }
                                Err(e) if e.kind() == DecodeErrorKind::FrameTooShort => {
                                    break;
                                }
                                Err(e) if e.kind() == DecodeErrorKind::FrameTooLong => {
                                    accumulator.clear();
                                    break;
                                }
                                Err(e) => {
                                    println!("Failed to decode frame: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Ok(_) => (),
                    Err(ref e) if e.kind() == ErrorKind::TimedOut => (),
                    Err(e) => {
                        error!("Serial read error: {}. Restarting connection...", e);
                        break; // Break inner loop to trigger reconnect
                    }
                }
                tokio::time::sleep(Duration::from_millis(SERIAL_LOOP_SLEEP_MS)).await;
            }
            
            error!("Serial loop inner loop exited. Attempting to reconnect in 1s...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    InvalidDelimiter,
    FrameTooShort,
    FrameTooLong,
    Misc,
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
            DecodeError::Misc(_) => DecodeErrorKind::Misc,
        }
    }
}

pub fn decode_frame(in_frame: &[u8]) -> Result<(SerialFrame, Option<Vec<u8>>), DecodeError> {
    let mut buffer: Vec<u8>;
    let mut vec_frame: Vec<u8>;
    let mut residual: Option<Vec<u8>> = None;
    if in_frame.len() < 4 {
        return Err(DecodeError::FrameTooShort);
    }
    if in_frame[0] != SERIAL_INBOUND && in_frame[0] != SERIAL_OUTBOUND {
        return Err(DecodeError::InvalidDelimiter);
    }
    let frame_length = u16::from_le_bytes([in_frame[1], in_frame[2]]);
    if in_frame.len() < frame_length as usize + 3 {
        return Err(DecodeError::FrameTooShort);
    }
    vec_frame = in_frame.to_vec();
    if in_frame.len() > frame_length as usize + 3 {
        buffer = vec_frame.drain(0..frame_length as usize + 3).collect();
        residual = Some(vec_frame);
    } else {
        buffer = vec_frame;
    }
    let delimiter = buffer[0];
    let frame_length = u16::from_le_bytes([buffer[1], buffer[2]]);
    let frame = buffer[3..].to_vec();
    Ok((
        SerialFrame {
            delimiter,
            frame_length,
            frame,
        },
        residual,
    ))
}
