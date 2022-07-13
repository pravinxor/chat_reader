#[path = "common.rs"]
mod common;

#[path = "afreecatv.rs"]
mod afreecatv;

#[path = "twitch.rs"]
mod twitch;

#[path = "tiktok.rs"]
mod tiktok;

use crate::common::Vod;
use clap::Parser;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    /// Load all live channels from tag and read chats from those channels
    #[clap(long, value_parser)]
    twitch_tag: Option<String>,

    /// Read chats from all videos within a channel
    #[clap(long, value_parser)]
    twitch_channels: Option<Vec<String>>,

    /// Read chat from a single video
    #[clap(long, value_parser)]
    twitch_vods: Option<Vec<u32>>,

    /// Read chats from all vods within a blog
    #[clap(long, value_parser)]
    afreecatv_channels: Option<Vec<String>>,

    /// Read chat from a single video
    #[clap(long, value_parser)]
    afreecatv_vods: Option<Vec<u32>>,

    /// Read comments from a single tiktok video
    #[clap(long, value_parser)]
    tiktok_vods: Option<Vec<u64>>,

    /// Filter chat search results
    #[clap(short, long, value_parser, default_value = "")]
    filter: String,

    #[clap(short, long, parse(from_flag))]
    showall: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let ftype = format!("(?i)({})", args.filter);
    let filter = regex::Regex::new(&ftype)?;

    if let Some(tag) = args.twitch_tag {
        let tag = crate::twitch::Tag::new(&tag)?;
        let channels = tag.channels()?;
        channels.iter().for_each(|channel| {
            println!("Working on {}", channel.username);
            crate::common::print_iter(&channel.videos().unwrap(), &filter, args.showall);
        });
    }

    if let Some(usernames) = args.twitch_channels {
        for username in usernames {
            let channel = crate::twitch::Channel::new(username);
            let videos = channel.videos()?;
            crate::common::print_iter(&videos, &filter, args.showall);
        }
    }

    if let Some(vods) = args.twitch_vods {
        for vod in vods {
            let vod = crate::twitch::Vod::new(vod);
            vod.comments()
                .flatten()
                .filter(|m| filter.is_match(&m.body))
                .for_each(|comment| println!("{}", comment));
        }
    }

    if let Some(usernames) = args.afreecatv_channels {
        for username in usernames {
            let channel = crate::afreecatv::Channel::new(username);
            let videos = channel.videos()?;
            crate::common::print_iter(&videos, &filter, args.showall);
        }
    }

    if let Some(vods) = args.afreecatv_vods {
        for vod in vods {
            let vod = crate::afreecatv::Vod::new(vod)?;
            vod.comments()
                .flatten()
                .filter(|m| filter.is_match(&m.body))
                .for_each(|comment| println!("{}", comment));
        }
    }

    if let Some(vods) = args.tiktok_vods {
        for vod in vods {
            let vod = crate::tiktok::Vod::new(vod);
            vod.comments()
                .flatten()
                .filter(|m| filter.is_match(&m.body))
                .for_each(|comment| println!("{}", comment));
        }
    };
    Ok(())
}
