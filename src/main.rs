#[path = "common.rs"]
mod common;

#[path = "twitch.rs"]
mod twitch;

use crate::common::Vod;
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Read chats from all video within a channel
    #[clap(long, value_parser)]
    twitch_channel: Option<String>,

    /// Read chat from a single video
    #[clap(long, value_parser)]
    twitch_vod: Option<u32>,

    /// Filter chat search results
    #[clap(short, long, value_parser, default_value = "")]
    filter: String,
}

fn main() {
    let args = Args::parse();

    let filter = regex::Regex::new(&args.filter).unwrap();

    if let Some(username) = args.twitch_channel {
        let channel = crate::twitch::Channel::new(username);
        let videos = channel.videos().unwrap();
        crate::common::print_iter(&videos);
    }

    if let Some(vod) = args.twitch_vod {
        let vod = crate::twitch::Vod::new(vod);
        vod.comments()
            .flatten()
            .filter(|m| filter.is_match(&m.body))
            .for_each(|comment| println!("{}", comment));
    }
}
