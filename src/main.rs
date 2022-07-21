#[path = "common.rs"]
mod common;

#[path = "afreecatv.rs"]
mod afreecatv;

#[path = "twitch.rs"]
mod twitch;

#[path = "twitchrecover.rs"]
mod twitchrecover;

#[path = "tiktok.rs"]
mod tiktok;

use crate::common::Vod;
use clap::Parser;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    /// Attempt to recover all recent videos on a twitch channel, returning their (possibly broken) m3u8 links
    #[clap(long, value_parser)]
    twitch_recover_channel: Option<String>,

    /// Load all live channels from tag and read chats from those channels
    #[clap(long, value_parser)]
    twitch_tags: Option<String>,

    /// Loads all live channels from a game directory and reads chat from those channels
    #[clap(long, value_parser)]
    twitch_directory: Option<String>,

    /// Read titles of twitch clips from a channel
    #[clap(long, value_parser)]
    twitch_clips: Option<String>,

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

    /// Read comments from a single tiktok video
    #[clap(long, value_parser)]
    tiktok_vod: Option<u64>,

    /// Filter chat search results
    #[clap(short, long, value_parser, default_value = "")]
    filter: regex::Regex,

    #[clap(short, long, parse(from_flag))]
    showall: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let filter = args.filter;

    if let Some(username) = args.twitch_recover_channel {
        let channel = crate::twitchrecover::Channel::new(&username).unwrap();
        channel.videos()?;
    }

    if let Some(directory) = args.twitch_directory {
        let directory = crate::twitch::Directory::new(directory);
        let channels = directory.channels()?;
        channels.iter().for_each(|channel| {
            println!("Working on {}", channel.username);
            crate::common::print_iter(&channel.videos().unwrap(), &filter, args.showall);
        });
    }

    if let Some(tag) = args.twitch_tags {
        let tags: Vec<&str> = tag.split_whitespace().collect();
        let tag = crate::twitch::Tag::new(&tags);
        let channels = tag.channels()?;
        channels.iter().for_each(|channel| {
            println!("Working on {}", channel.username);
            crate::common::print_iter(&channel.videos().unwrap(), &filter, args.showall);
        });
    }

    if let Some(username) = args.twitch_clips {
        let channel = crate::twitch::Channel::new(username);
        let clips = channel.clips();
        clips
            .flatten()
            .filter(|c| filter.is_match(c.user.as_ref().unwrap()) || filter.is_match(&c.body))
            .for_each(|c| println!("{}", c));
    }

    if let Some(username) = args.twitch_channel {
        let channel = crate::twitch::Channel::new(username);
        let videos = channel.videos()?;
        crate::common::print_iter(&videos, &filter, args.showall);
    }

    if let Some(vod) = args.twitch_vod {
        let vod = crate::twitch::Vod::new(vod);
        vod.comments()
            .flatten()
            .filter(|m| filter.is_match(&m.body) || filter.is_match(m.user.as_ref().unwrap()))
            .for_each(|comment| println!("{}", comment));
    }

    if let Some(username) = args.afreecatv_channel {
        let channel = crate::afreecatv::Channel::new(username);
        let videos = channel.videos()?;
        crate::common::print_iter(&videos, &filter, args.showall);
    }

    if let Some(vod) = args.afreecatv_vod {
        let vod = crate::afreecatv::Vod::new(vod)?;
        vod.comments()
            .flatten()
            .filter(|m| filter.is_match(&m.body) || filter.is_match(m.user.as_ref().unwrap()))
            .for_each(|comment| println!("{}", comment));
    }

    if let Some(vod) = args.tiktok_vod {
        let vod = crate::tiktok::Vod::new(vod);
        vod.comments()
            .flatten()
            .filter(|m| filter.is_match(&m.body) || filter.is_match(m.user.as_ref().unwrap()))
            .for_each(|comment| println!("{}", comment));
    };
    Ok(())
}
