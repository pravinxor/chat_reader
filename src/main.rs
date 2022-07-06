#[path = "common.rs"]
mod common;

#[path = "afreecatv.rs"]
mod afreecatv;

#[path = "twitch.rs"]
mod twitch;

use crate::common::Vod;
use clap::Parser;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    /// Read chats from all videos within a channel
    #[clap(long, value_parser)]
    twitch_channel: Option<String>,

    /// Read chat from a single video
    #[clap(long, value_parser)]
    twitch_vod: Option<u32>,

    /// Read chats from all vods within a blog
    #[clap(long, value_parser)]
    afreecatv_channel: Option<String>,

    /// Read chat from a single video
    #[clap(long, value_parser)]
    afreecatv_vod: Option<u32>,

    /// Filter chat search results
    #[clap(short, long, value_parser, default_value = "")]
    filter: String,
}

fn main() {
    let args = Args::parse();
    let ftype = format!("(?i)({})", args.filter);
    let filter = regex::Regex::new(&ftype).unwrap();

    if let Some(username) = args.twitch_channel {
        let channel = crate::twitch::Channel::new(username);
        let videos = channel.videos().unwrap();
        crate::common::print_iter(&videos, &filter);
    }

    if let Some(vod) = args.twitch_vod {
        let vod = crate::twitch::Vod::new(vod);
        vod.comments()
            .flatten()
            .filter(|m| filter.is_match(&m.body))
            .for_each(|comment| println!("{}", comment));
    }

    if let Some(username) = args.afreecatv_channel {
        let channel = crate::afreecatv::Channel::new(username);
        let videos = channel.videos().unwrap();
        crate::common::print_iter(&videos, &filter);
    }

    if let Some(vod) = args.afreecatv_vod {
        let vod = crate::afreecatv::Vod::new(vod).unwrap();
        vod.comments()
            .flatten()
            .filter(|m| filter.is_match(&m.body))
            .for_each(|comment| println!("{}", comment));
    }
}
