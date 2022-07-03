#[path = "common.rs"]
mod common;

#[path = "twitch.rs"]
mod twitch;

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Read chats from all video within a channel
    #[clap(long, value_parser)]
    twitch_channel: Option<String>,

    /// Read chat from a single video
    #[clap(long, value_parser)]
    twitch_vod: Option<u32>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(username) = args.twitch_channel {
        let channel = crate::twitch::Channel::new(username);
        dbg!(channel.videos().await.unwrap());
    }

    if let Some(vod) = args.twitch_vod {
        let vod = crate::twitch::Vod::new(vod);
        vod.comments().flatten().for_each(|message| println!("{}", message));
    }
}
