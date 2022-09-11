lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

pub const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.5060.114 Safari/537.36";

use hhmmss::Hhmmss;

#[derive(Debug)]
pub struct Message {
    pub user: Option<String>,
    pub body: String,
    pub timestamp: Option<f64>,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(seconds) = self.timestamp {
            let seconds = std::time::Duration::from_secs(seconds as u64);
            write!(f, "[{}]", seconds.hhmmss())?
        }
        if let Some(user) = &self.user {
            write!(f, "[{}]", user)?
        }
        write!(f, " {}", self.body)
    }
}

pub trait Vod: std::fmt::Display {
    fn comments(&self) -> Box<dyn ChatIterator>;
}

pub trait ChatIterator: Send + Iterator<Item = Vec<Message>> {}

pub fn print_iter<V>(vods: &[V], filter: &regex::Regex, showall: bool, sequence: &oqueue::Sequencer)
where
    V: Vod + Sync,
{
    rayon::scope_fifo(|t| {
        for vod in vods {
            t.spawn_fifo(|_| {
                let comments = vod.comments().flatten();
                let mut task = sequence.begin();
                if !showall {
                    task.hold();
                }
                writeln!(task, "{}", vod.to_string());
                for comment in comments.filter(|message| {
                    filter.is_match(&message.body)
                        || match message.user.as_ref() {
                            Some(message) => filter.is_match(message),
                            None => false,
                        }
                }) {
                    task.release();
                    writeln!(task, "{}", comment);
                }
                writeln!(task);
            });
        }
    });
}
