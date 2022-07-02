#[path = "common.rs"]
mod common;

#[path = "twitch.rs"]
mod twitch;

#[tokio::main]
async fn main() {
    let channel = twitch::Channel::new("twitch");
    let vods = channel.videos().await.unwrap();
    dbg!(vods);
}
