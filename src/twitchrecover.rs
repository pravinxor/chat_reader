use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha1::Digest;

const CLOUDFRONT_DOMAINS: [&str; 9] = [
    "d2e2de1etea730",
    "dqrpb9wgowsf5",
    "ds0h3roq6wcgc",
    "d2nvs31859zcd8",
    "d2aba1wr3818hz",
    "d3c27h4odz752x",
    "dgeft87wbj63p",
    "d1m7jfoe9zdc1j",
    "d1ymi26ma8va5x",
];

#[derive(Debug)]
pub struct Channel {
    value: u64,
}

#[derive(Debug)]
pub struct Video {
    id: u64,
    title: Option<String>,
    link: String,
}

impl Channel {
    pub fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json: serde_json::Value = crate::common::CLIENT
            .get(format!(
                "https://sullygnome.com/api/standardsearch/{}",
                name
            ))
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
            .send()?
            .json()?;

        let top_value = json
            .get(0)
            .ok_or("Missing idx 0, No results found")?
            .get("value")
            .ok_or("Missing value")?
            .as_u64()
            .ok_or("Could not convert value -> u64")?;

        Ok(Self { value: top_value })
    }

    pub fn unix_time(time: &str) -> Result<i64, chrono::ParseError> {
        Ok(chrono::NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M:%SZ")?.timestamp())
    }

    pub fn videos(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json: serde_json::Value = crate::common::CLIENT
            .get(&format!(
                "https://sullygnome.com/api/tables/channeltables/streams/365/{}/%20/1/1/desc/0/100",
                self.value
            ))
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
            .send()?
            .json()?;
        let data = json
            .get("data")
            .ok_or("Missing data")?
            .as_array()
            .ok_or("Could not convert data -> array")?;
        let sequencer = oqueue::Sequencer::stdout();
        rayon::scope_fifo(|t| {
            for video in data {
                t.spawn_fifo(|_| {
                    let task = sequencer.begin();
                    let rvideo = |video: &serde_json::Value| -> Option<Video> {
                        let stream_id = video.get("streamId")?.as_u64()?;
                        let start_timestamp = video.get("startDateTime")?.as_str()?;
                        let unix_timestamp = Self::unix_time(start_timestamp).unwrap();
                        let channel_name = video.get("channelurl")?.as_str()?;
                        let starttime = video
                            .get("starttime")
                            .map(|s| s.as_str().unwrap().to_owned());
                        Video::new(stream_id, unix_timestamp, channel_name, starttime)
                    };
                    if let Some(video) = rvideo(video) {
                        writeln!(task, "{}\n", video);
                    }
                });
            }
        });
        Ok(())
    }
}

impl std::fmt::Display for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.id)?;
        if let Some(title) = &self.title {
            write!(f, " [{}]", title)?;
        }
        write!(f, "\n{}", self.link)
    }
}

impl Video {
    pub fn new(
        stream_id: u64,
        timestamp: i64,
        channel_name: &str,
        title: Option<String>,
    ) -> Option<Self> {
        let body = format!("{}_{}_{}", channel_name, stream_id, timestamp);
        let hash = format!("{:x}", sha1::Sha1::digest(&body));
        let subdirectory = format!("{}_{}", &hash[0..20], body);

        let cloudfront_link = CLOUDFRONT_DOMAINS
            .par_iter()
            .flat_map(|domain| {
                let link = format!(
                    "https://{}.cloudfront.net/{}/chunked/index-dvr.m3u8",
                    domain, &subdirectory
                );
                let request = crate::common::CLIENT.get(&link).send();
                if let Ok(message) = request {
                    if message.status().is_success() {
                        Some(link)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .find_any(|_| true);

        cloudfront_link.map(|link| Self {
            id: stream_id,
            link,
            title,
        })
    }
}
