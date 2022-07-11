lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

use hhmmss::Hhmmss;
use std::io::Write;

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
        finish_sig: std::sync::mpsc::Sender<()>,
        display_sig: std::sync::mpsc::Receiver<()>,
        filter: &regex::Regex,
    ) {
        let mut display_now = false;
        let mut buf = Vec::new();
        let mut writer = std::io::BufWriter::new(std::io::stdout());
        for message in self
            .flatten()
            .filter(|message| filter.is_match(&message.body))
        {
            buf.push(message);
            if display_now {
                buf.iter().for_each(|m| writeln!(writer, "{}", m).unwrap());
                buf.clear();
            } else if display_sig.try_recv().is_ok() {
                display_now = true;
            }
        }
        if !display_now {
            display_sig.recv().unwrap();
            buf.iter().for_each(|m| println!("{}", m));
        }
        writer.flush().unwrap();
        finish_sig.send(()).unwrap();
    }
}

pub fn print_iter<V>(vods: &[V], filter: &regex::Regex)
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
            t.spawn(move |_| comments.display_worker(ftx, rx, filter));
            future_manager.push((vod, frx, tx));
        }

        for (vod, frx, tx) in future_manager {
            println!("{}", vod);
            tx.send(()).unwrap();
            frx.recv().unwrap();
            println!();
        }
    });
}
