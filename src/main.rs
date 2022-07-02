#[path = "common.rs"]
mod common;

#[path = "twitch.rs"]
mod twitch;

#[tokio::main]
async fn main() {
    let channel = twitch::Channel::new("twitch");
    channel.videos().await;
}
