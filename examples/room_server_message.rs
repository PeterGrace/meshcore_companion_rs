#[macro_use]
extern crate tracing;
use console_subscriber as tokio_console_subscriber;
use meshcore_companion_rs::commands::{
    DeviceQuery, GetContacts, LoginData, SendChannelTxtMsg, SendTxtMsg,
};
use meshcore_companion_rs::consts;
use meshcore_companion_rs::consts::CMD_SEND_LOGIN;
use meshcore_companion_rs::contact_mgmt::PublicKey;
use meshcore_companion_rs::{string_to_bytes, AppStart, Commands, Companion, MessageTypes};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[tokio::main]
async fn main() {
    let default_log_level = "info".to_string();
    //region console logging
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
        .with_span_events(FmtSpan::NONE);

    let subscriber = Registry::default()
        .with(console_layer)
        .with(filter_layer)
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
    let _ = companion.command(Commands::CmdGetBattAndStorage).await;
    let _ = companion.command(Commands::CmdSetDeviceTime).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let _ = companion.command(Commands::CmdGetDeviceTime).await;

    //endregion

    // Give the companion a moment to initialize and download contacts list
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let roomsrv_key: PublicKey =
        PublicKey::from_hex("2c4bd0601028f9876be8795d94a5ca1f9f798d3eb59d124985d90928ffc6e155")
            .expect("Couldn't convert hex to key");

    let login: LoginData = LoginData {
        code: CMD_SEND_LOGIN,
        public_key: roomsrv_key,
        password: string_to_bytes::<15>("hello"),
    };
    // 3c 27 00 1a 2c 4b d0 60  10 28 f9 87 6b e8 79 5d
    // 94 a5 ca 1f 9f 79 8d 3e  b5 9d 12 49 85 d9 09 28
    // ff c6 e1 55 66 30 30 62  34 72                                             
    if let Err(e) = companion
        .command(Commands::CmdSendLogin(login.clone()))
        .await
    {
        warn!("Sending login command failed: {e:?}");
    }
    let mut logged_in = false;
    while !logged_in {
        info!("Checking login status...");
        let contact = companion
            .find_contact_by_key_prefix(roomsrv_key.prefix())
            .await
            .unwrap();
        if let Some(status) = contact.logged_in {
            if status {
                logged_in = true;
            } else {
                panic!("Login failed!");
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let msg = SendTxtMsg {
        code: consts::CMD_SEND_TXT_MSG,
        txt_type: 0,
        attempt: 0,
        sender_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,
        pubkey_prefix: roomsrv_key.prefix_bytes(),
        text: "Test Send To Room Server!".to_string(),
        timeout: None,
    };
    let _ = companion.command(Commands::CmdSendTxtMsg(msg)).await;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //info!("Message sent! Logging out!");
    //let _ = companion.command(Commands::CmdLogout(contact.public_key)).await;

    info!("Press Ctrl+C to exit");

    // Receive messages
    loop {
        while let Some(msg) = companion.pop_message().await {
            match msg {
                MessageTypes::ContactMsg(msg) => {
                    info!(
                        "[{}] {}",
                        msg.pubkey_prefix,
                        msg.text
                    );
                }
                MessageTypes::ContactMsgV3(msg) => {
                    info!(
                        "[{}] {}",
                        msg.pubkey_prefix,
                        msg.text
                    );
                }
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
