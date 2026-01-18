#[macro_use] extern crate tracing;
use tracing_subscriber::{EnvFilter, Layer, Registry};
use tracing_subscriber::fmt::format::FmtSpan;
use console_subscriber as tokio_console_subscriber;
use tracing_subscriber::layer::SubscriberExt;
use meshcore_companion_rs::{Companion, Commands, MessageTypes, AppStart};
use meshcore_companion_rs::commands::{DeviceQuery, GetContacts, SendChannelTxtMsg, SendTxtMsg};
use meshcore_companion_rs::consts;

#[tokio::main]
async fn main() {
    //region console logging
    let default_log_level = "info".to_string();
    let console_layer = tokio_console_subscriber::spawn();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_log_level))
        .unwrap();
    let format_layer = tracing_subscriber::fmt::layer()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_line_number(true),
        )
        .with_span_events(FmtSpan::NONE)
        .with_filter(filter_layer);

    let subscriber = Registry::default()
        .with(console_layer)
        .with(format_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
    //endregion
    let mut companion = Companion::new("/dev/ttyUSB0");
    companion.start().await.unwrap();

    //region companion data sync (contacts, app info)
    let appstart: AppStart = AppStart {
        code: consts::CMD_APP_START,
        app_ver: 1,
        app_name: "test".to_string(),
        ..AppStart::default()
    };
    let _ = companion.command(Commands::CmdAppStart(appstart)).await;

    let data: DeviceQuery = DeviceQuery {
        code: consts::CMD_DEVICE_QEURY,
        app_target_ver: 3,
    };
    let _ = companion.command(Commands::CmdDeviceQuery(data)).await;

    let data: GetContacts = GetContacts {
        code: consts::CMD_GET_CONTACTS,
        since: None,
    };
    let _ = companion.command(Commands::CmdGetContacts(data)).await;
    //endregion

    // Give the companion a moment to initialize and download contacts list
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // G2GH    = 0663f1725334df8c20b2269e426c546ca9bea2a975c287eb1bcead3cdac56fb6
    // Pete    = 0680ae32618ef25b6a43b30c646d8458f6da82c33556a8ced1600aa111588b6f
    // RoomSrv = 2c4bd0601028f9876be8795d94a5ca1f9f798d3eb59d124985d90928ffc6e155
    // pete    = 4d10b03a615e15f703f85d471251c61625745a051fd49ecfe3efce7e2a86d50b
    // test sending a message to contact
    if let Some(contact) = companion.find_contact("Pete").await {
        info!("found contact: {:?}", contact);
        let pubkey_prefix: [u8; 6] = <[u8; 6]>::try_from(contact.public_key.prefix()).unwrap();
        let msg = SendTxtMsg {
            code: consts::CMD_SEND_TXT_MSG,
            txt_type: 0,
            attempt: 0,
            sender_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32,
            pubkey_prefix,
            text: "Private Message!".to_string(),
            timeout: None
        };
        let _ = companion.command(Commands::CmdSendTxtMsg(msg)).await;
        info!("Message sent! Listening for incoming messages...");
    } else {
        error!("contact not found");
    }


    info!("Press Ctrl+C to exit");

    // Receive messages
    loop {
        while let Some(msg) = companion.pop_message().await {
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
