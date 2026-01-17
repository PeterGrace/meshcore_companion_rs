#[macro_use]
extern crate tracing;

use console_subscriber as tokio_console_subscriber;
use meshcore_companion_rs::{Commands, MessageTypes};
use meshcore_companion_rs::commands::{DeviceQuery, GetContacts, SendChannelTxtMsg};
use meshcore_companion_rs::consts;
use meshcore_companion_rs::{AppStart, Companion};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

#[tokio::main]
pub async fn main() {
    //region console logging
    let console_layer = tokio_console_subscriber::spawn();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let format_layer = tracing_subscriber::fmt::layer()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_line_number(true),
        )
        .with_span_events(FmtSpan::NONE);

    let subscriber = Registry::default()
        .with(console_layer)
        .with(filter_layer)
        .with(format_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
    //endregion

    info!("companion test app starting");
    let mut foo = Companion::new("/dev/ttyUSB0");
    foo.listen().unwrap();
    let appstart: AppStart = AppStart {
        code: consts::CMD_APP_START,
        app_ver: 1,
        app_name: "test".to_string(),
        ..AppStart::default()
    };
    let _ = foo.command(Commands::CmdAppStart(appstart)).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let data: DeviceQuery = DeviceQuery {
        code: consts::CMD_DEVICE_QEURY,
        app_target_ver: 3,
    };
    let _ = foo.command(Commands::CmdDeviceQuery(data)).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let data: GetContacts = GetContacts {
        code: consts::CMD_GET_CONTACTS,
        since: None,
    };
    let _ = foo.command(Commands::CmdGetContacts(data)).await;

    let msg = SendChannelTxtMsg {
        code: consts::CMD_SEND_CHANNEL_TXT_MSG,
        txt_type: 0,
        channel_idx: 0,
        sender_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,        text: "Hello World!".to_string(),
    };
    let _ = foo.command(Commands::CmdSendChannelTxtMsg(msg)).await;


    loop {
        foo.check().await.unwrap();
        while let Some(msg) = foo.pending_messages.pop() {
            match msg {
                MessageTypes::ContactMsg(msg) => {
                    info!("[{}] {}", msg.pubkey_prefix.iter().map(|b| format!("{:02x}", b)).collect::<String>(), msg.text);
                },
                MessageTypes::ContactMsgV3(msg) => {
                    info!("[{}] {}", msg.pubkey_prefix.iter().map(|b| format!("{:02x}", b)).collect::<String>(), msg.text);
                },
                MessageTypes::ChannelMsg(msg) => {
                    info!("[{}] {}", msg.channel_id, msg.text);
                }
                MessageTypes::ChannelMsgV3(msg) => {
                    info!("[{}] {}", msg.channel_id, msg.text);
                }

            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }
}
