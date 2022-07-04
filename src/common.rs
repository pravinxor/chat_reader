pub const TWITCH_CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

use colored::Colorize;
use hhmmss::Hhmmss;

#[derive(Debug)]
pub struct Message {
    pub user: String,
    pub color: colored::Color,
    pub body: String,
    pub timestamp: f64,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let coloreduser = self.user.color(self.color);
        let seconds = std::time::Duration::from_secs(self.timestamp as u64);
        write!(f, "[{}][{}]: {}", seconds.hhmmss(), coloreduser, self.body)
    }
}

pub trait ChatIterator: Iterator<Item = Vec<Message>> {}

pub trait Vod {
    fn comments(&self) -> Box<dyn ChatIterator>;
}

pub fn print_iter<V>(messages: &[V])
where
    V: Vod,
{
}
