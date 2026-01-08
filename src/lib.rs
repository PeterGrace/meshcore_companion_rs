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
use thiserror::Error;
use tokio::sync::mpsc;
pub use crate::commands::{AppStart, Commands};
use crate::responses::Responses;
use crate::serial_actor::{SerialFrame, serial_loop};
use crate::commands::Reboot;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Misc: {0}")]
    Misc(String),
}

pub struct Companion {
    to_radio_tx: mpsc::Sender<SerialFrame>,
    to_radio_rx: Option<mpsc::Receiver<SerialFrame>>,
    from_radio_tx: mpsc::Sender<SerialFrame>,
    from_radio_rx: mpsc::Receiver<SerialFrame>,
    receive_queue: HashMap<u8, Vec<u8>>,
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
            port: port.to_string() 
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
            info!("Received message: {:?}", msg);

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
                let data = vec![];
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