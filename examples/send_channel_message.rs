#[macro_use] extern crate tracing;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::fmt::format::FmtSpan;
use console_subscriber as tokio_console_subscriber;
use tracing_subscriber::layer::SubscriberExt;
use meshcore_companion_rs::{Companion, Commands};
use meshcore_companion_rs::commands::SendChannelTxtMsg;
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
        .with_span_events(FmtSpan::NONE);

    let subscriber = Registry::default()
        .with(console_layer)
        .with(filter_layer)
        .with(format_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
    //endregion
    let mut companion = Companion::new("/dev/ttyUSB0");
    companion.start().await.unwrap();

    // Give the companion a moment to initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Send a channel message
    let msg = SendChannelTxtMsg {
        code: consts::CMD_SEND_CHANNEL_TXT_MSG,
        txt_type: 0,
        channel_idx: 0,
        sender_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,
        text: "Hello World!".to_string(),
    };
    companion.command(Commands::CmdSendChannelTxtMsg(msg)).await.unwrap();

    info!("Message sent! Listening for incoming messages...");
    info!("Press Ctrl+C to exit");

    // Receive messages
    loop {
        while let Some(msg) = companion.pop_message().await {
            info!("Received message: {:?}", msg);
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        }
    }
}
