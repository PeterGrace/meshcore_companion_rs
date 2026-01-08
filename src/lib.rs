#[macro_use] extern crate tracing;
pub mod consts;
pub mod commands;
pub mod push_events;
pub mod responses;

mod serial_actor;
mod tests;

use std::collections::HashMap;
use tokio::task::JoinHandle;
use std::io::Read;
use lazy_static::lazy_static;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
pub use crate::commands::{AppStart, Commands};
use crate::responses::{DeviceInfo, Responses};
use crate::serial_actor::{SerialFrame, serial_loop};
use crate::commands::Reboot;
use crate::responses::SelfInfo;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Misc: {0}")]
    Misc(String),
}

#[derive(Debug)]
pub struct Companion {
    to_radio_tx: mpsc::Sender<SerialFrame>,
    to_radio_rx: Option<mpsc::Receiver<SerialFrame>>,
    from_radio_tx: mpsc::Sender<SerialFrame>,
    from_radio_rx: mpsc::Receiver<SerialFrame>,
    receive_queue: HashMap<u8, Responses>,
    self_info: Option<SelfInfo>,
    device_info: Option<DeviceInfo>,
    port: String
}

impl Companion {
    pub fn new(port: &str) -> Self {
        let (to_radio_tx, to_radio_rx) = mpsc::channel(consts::MPSC_BUFFER_DEPTH);
        let (from_radio_tx, from_radio_rx) = mpsc::channel(consts::MPSC_BUFFER_DEPTH);
        Companion { 
            to_radio_tx, 
            to_radio_rx: Some(to_radio_rx), 
            from_radio_tx, 
            from_radio_rx,
            receive_queue: HashMap::new(),
            port: port.to_string(),
            self_info: None,
            device_info: None
        }
    }
    pub fn listen(&mut self) -> Result<JoinHandle<()>, AppError> {
        let port = self.port.clone();
        let from_radio_tx = self.from_radio_tx.clone();
        let mut to_radio_rx = self.to_radio_rx.take().ok_or_else(|| {
            AppError::Misc("Listener already started".to_string())
        })?;

        Ok(tokio::spawn(async move {
            serial_loop(port, &mut to_radio_rx, &from_radio_tx).await;
        }))
    }
    pub fn check(&mut self) -> Result<(), AppError> {
        while let Ok(msg) = self.from_radio_rx.try_recv() {
            let frame = msg.frame;
            match frame[0] {
                consts::RESP_CODE_SELF_INFO => {
                    let self_info = SelfInfo::from_frame(&frame);
                    debug!("Received self info response: {self_info:#?}");
                    self.self_info = Some(self_info);
                }
                consts::RESP_CODE_DEVICE_INFO => {
                    let device_info = DeviceInfo::from_frame(&frame);
                    debug!("Received device info response: {device_info:#?}");
                    self.device_info = Some(device_info);
                },
                _ => {
                    warn!("unimplemented response code: {:02x} {:02x?}", frame[0], frame);
                },
            }

        }
        Ok(())

    }
    pub async fn command(&mut self, cmd: Commands) -> Result<Responses, AppError> {
        match cmd {
            Commands::CmdReboot => {
                let data: Vec<u8> = vec![0x13,0x72,0x65,0x62,0x6f,0x6f,0x74];
                let frame: SerialFrame = SerialFrame {
                    delimiter: consts::SERIAL_OUTBOUND,
                    frame_length: data.len() as u16,
                    frame: data
                };
                self.to_radio_tx.send(frame).await.unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));
                Err(AppError::Misc("Not implemented yet".to_string()))
            }
            Commands::CmdAppStart(app) => {
                // Send command
                let data: Vec<u8> = vec![consts::CMD_APP_START, 0x03,0x00,0x00,0x00,0x00,0x00,0x00,0x01];

                let frame: SerialFrame = SerialFrame {
                    delimiter: consts::SERIAL_OUTBOUND,
                    frame_length: data.len() as u16,
                    frame: data
                };
                self.to_radio_tx.send(frame).await.unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));

                // Try to receive response
                Err(AppError::Misc("Not implemented yet".to_string()))
            },
            Commands::CmdDeviceQuery(app) => {
                // Send command
                let data = vec![consts::CMD_DEVICE_QEURY, app.app_target_ver];
                let frame: SerialFrame = SerialFrame {
                    delimiter: consts::SERIAL_OUTBOUND,
                    frame_length: data.len() as u16,
                    frame: data
                };
                self.to_radio_tx.send(frame).await.unwrap_or_else(|e| error!("Failed to send serial frame: {}", e));

                // Try to receive response
                Err(AppError::Misc("Not implemented yet".to_string()))
            },
                _ => todo!(),
            }

    }
}