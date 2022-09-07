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
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Args {
    #[clap(subcommand)]
    mode: Mode,

    /// Filter chat search results
    #[clap(short, long, value_parser, default_value = "")]
    filter: regex::Regex,
}

#[derive(clap::Args)]
#[clap(arg_required_else_help(true))]
struct TwitchChannelOpts {
    /// Read all clips in a channel and returns matches
    #[clap(short, long, parse(from_flag))]
    clips: bool,

    /// Read all vods in a channel and return transcript sections with matches
    #[clap(short, long, parse(from_flag))]
    vods: bool,

    /// Attempt to recover as many (vod-pointing) m3u8 links (upto 365 days prior) from a channel
    #[clap(short, long, parse(from_flag))]
    recover: bool,

    #[clap(short, long, parse(from_flag))]
    showall: bool,
}

#[derive(Subcommand)]
enum DirectoryClips {
    DirectoryClips { recency: crate::twitch::Recency },
}

#[derive(Subcommand)]
enum Twitch {
    Vod {
        id: u32,
    },
    Channel {
        channel: crate::twitch::Channel,

        #[clap(flatten)]
        opts: TwitchChannelOpts,
    },
    Directory {
        name: String,

        /// Retrieves the clips associated with directory, which can be from channels that are no
        /// longer live
        #[clap(subcommand)]
        directory_clips: Option<DirectoryClips>,

        /// Channel Options
        #[clap(flatten)]
        opts: TwitchChannelOpts,
    },
    Tags {
        tags: Vec<String>,

        /// Channel Options
        #[clap(flatten)]
        opts: TwitchChannelOpts,
    },
}

#[derive(Subcommand)]
enum Afreecatv {
    Vod {
        id: u32,
    },
    Blog {
        username: String,

        #[clap(short, long, parse(from_flag))]
        showall: bool,
    },
}

#[derive(Subcommand)]
enum TikTok {
    Video {
        id: u64,
        comments: bool,
        transcript: bool,
    },
}

#[derive(Subcommand)]
enum Mode {
    Twitch {
        #[clap(subcommand)]
        twitch: Twitch,
    },

    Afreecatv {
        #[clap(subcommand)]
        afreecatv: Afreecatv,
    },

    Tiktok {
        #[clap(subcommand)]
        tiktok: TikTok,
    },
}

fn handle_twitch_channel(
    channel: crate::twitch::Channel,
    opts: &TwitchChannelOpts,
    filter: &regex::Regex,
) -> Result<(), Box<dyn std::error::Error>> {
    if opts.clips {
        channel
            .clips()
            .flatten()
            .filter(|c| filter.is_match(c.user.as_ref().unwrap()) || filter.is_match(&c.body))
            .for_each(|c| println!("{}", c));
        println!();
    }
    if opts.vods {
        let videos = channel.videos()?;
        crate::common::print_iter(&videos, filter, opts.showall);
    }
    if opts.recover {
        let channel = crate::twitchrecover::Channel::new(&channel.username).unwrap();
        channel.videos()?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let filter = args.filter;

    match args.mode {
        Mode::Twitch { twitch } => match twitch {
            Twitch::Vod { id } => {
                let vod = crate::twitch::Vod::new(id);
                vod.comments()
                    .flatten()
                    .filter(|m| {
                        filter.is_match(&m.body) || filter.is_match(m.user.as_ref().unwrap())
                    })
                    .for_each(|comment| println!("{}", comment));
            }

            Twitch::Channel { channel, opts } => handle_twitch_channel(channel, &opts, &filter)?,

            Twitch::Directory {
                name,
                directory_clips,
                opts,
            } => {
                let directory = crate::twitch::Directory::new(&name);

                if let Some(DirectoryClips::DirectoryClips { recency }) = directory_clips {
                    directory
                        .clips(recency)
                        .flatten()
                        .filter(|c| {
                            filter.is_match(c.user.as_ref().unwrap()) || filter.is_match(&c.body)
                        })
                        .for_each(|c| println!("{}", c));
                }

                for channel in directory.channels().flatten() {
                    println!("Working on {}", channel.username.bold());
                    handle_twitch_channel(channel, &opts, &filter)?
                }
            }

            Twitch::Tags { tags, opts } => {
                for channel in crate::twitch::Tag::channels(&tags).flatten() {
                    println!("Working on {}", channel.username.bold());
                    handle_twitch_channel(channel, &opts, &filter)?
                }
            }
        },

        Mode::Afreecatv { afreecatv } => match afreecatv {
            Afreecatv::Vod { id } => {
                let vod = crate::afreecatv::Vod::new(id)?;
                vod.comments()
                    .flatten()
                    .filter(|m| {
                        filter.is_match(&m.body) || filter.is_match(m.user.as_ref().unwrap())
                    })
                    .for_each(|comment| println!("{}", comment));
            }

            Afreecatv::Blog { username, showall } => {
                let channel = crate::afreecatv::Channel::new(username);
                let videos = channel.videos()?;
                crate::common::print_iter(&videos, &filter, showall);
            }
        },

        Mode::Tiktok { tiktok } => match tiktok {
            TikTok::Video {
                id,
                comments,
                transcript,
            } => {
                let video = crate::tiktok::Vod::new(id);
                if comments {
                    video
                        .comments()
                        .flatten()
                        .filter(|m| {
                            filter.is_match(&m.body) || filter.is_match(m.user.as_ref().unwrap())
                        })
                        .for_each(|comment| println!("{}", comment));
                }
                if transcript {
                    video
                        .captions()
                        .flatten()
                        .filter(|m| filter.is_match(&m.body))
                        .for_each(|marker| println!("{}", marker));
                }
            }
        },
    }
    Ok(())
}
