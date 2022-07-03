#[path = "common.rs"]
mod common;

#[path = "twitch.rs"]
mod twitch;

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Twitch channel username
    #[clap(short, long, value_parser)]
    channel_twitch: Option<String>,

    /// Twitch Video ID
    #[clap(short, long, value_parser)]
    video_twitch: Option<u32>,

}

#[tokio::main]
async fn main() {
    //let channel = twitch::Channel::new("alinity");
    //let vods = channel.videos().await.unwrap();
    
    let args = Args::parse();

    if let Some(username) = args.channel_twitch {
        let channel = crate::twitch::Channel::new(username);
        dbg!(channel.videos().await.unwrap());
    }

    if let Some(vod) = args.video_twitch {
        let vod = crate::twitch::Vod::new(vod);
        for chunk in vod.comments() {
            dbg!(chunk);
        }
    }
}
