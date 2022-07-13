lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

use hhmmss::Hhmmss;

#[derive(Debug)]
pub struct Message {
    pub user: String,
    pub body: String,
    pub timestamp: Option<f64>,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(seconds) = self.timestamp {
            let seconds = std::time::Duration::from_secs(seconds as u64);
            write!(f, "[{}]", seconds.hhmmss())?
        }
        write!(f, "[{}]: {}", self.user, self.body)
    }
}

pub trait Vod: std::fmt::Display {
    fn comments(&self) -> Box<dyn ChatIterator>;
}

pub trait ChatIterator: Send + Iterator<Item = Vec<Message>> {
    /// Will walk the ChatIterator and save the output into a buffer.
    /// When display_sig recieves a signal, the buffer will be flushed into stdout
    fn display_worker(
        &mut self,
        ready_sig: std::sync::mpsc::Sender<bool>,
        finish_sig: std::sync::mpsc::Sender<()>,
        display_sig: std::sync::mpsc::Receiver<()>,
        filter: &regex::Regex,
        showall: bool,
    ) {
        let mut display_now = false;
        let mut has_messages = false;
        let mut buf = Vec::new();
        for message in self
            .flatten()
            .filter(|message| filter.is_match(&message.body))
        {
            if !showall && !has_messages {
                ready_sig.send(true).unwrap();
                has_messages = true;
            }

            buf.push(message);
            if display_now {
                buf.iter().for_each(|m| println!("{}", m));
                buf.clear();
            } else if display_sig.try_recv().is_ok() {
                display_now = true;
            }
        }
        if !showall && !has_messages {
            ready_sig.send(false).unwrap();
        }
        if !display_now {
            display_sig.recv().unwrap();
            buf.iter().for_each(|m| println!("{}", m));
        }
        finish_sig.send(()).unwrap();
    }
}

pub fn print_iter<V>(vods: &[V], filter: &regex::Regex, showall: bool)
where
    V: Vod + Sync,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(100)
        .build()
        .unwrap();
    pool.scope(|t| {
        let mut future_manager = Vec::with_capacity(vods.len());

        for vod in vods {
            let mut comments = vod.comments();
            let (tx, rx) = std::sync::mpsc::channel();
            let (ftx, frx) = std::sync::mpsc::channel();
            let (rtx, rrx) = std::sync::mpsc::channel();
            t.spawn(move |_| comments.display_worker(rtx, ftx, rx, filter, showall));

            future_manager.push((vod, rrx, frx, tx));
        }

        for (vod, rrx, frx, tx) in future_manager {
            let res = if !showall { rrx.recv().unwrap() } else { true };

            if res {
                println!("{}", vod);
            }
            tx.send(()).unwrap();
            frx.recv().unwrap();
            if res {
                println!();
            }
        }
    });
}
