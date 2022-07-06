use rayon::prelude::*;

#[derive(Debug)]
pub struct Vod {
    title_no: u32,
    station_no: u32,
    bbs_no: u32,
}

lazy_static::lazy_static! {
    static ref TITLE_NO_MATCHER: regex::Regex = regex::Regex::new(r#"document\.nTitleNo = [0-9]+;"#).unwrap();
    static ref STATION_NO_MATCHER: regex::Regex = regex::Regex::new(r#"document\.nStationNo = [0-9]+;"#).unwrap();
    static ref BBS_NO_MATCHER: regex::Regex = regex::Regex::new(r#"document\.nBbsNo = [0-9]+;"#).unwrap();

    static ref KEY_MATCHER: regex::Regex = regex::Regex::new(r#"key="([0-9]|[A-Z]|_)+""#).unwrap();
    static ref DURATION_MATCHER: regex::Regex = regex::Regex::new(r#"file duration="[0-9]+""#).unwrap();
}

impl Vod {
    pub fn new(title_no: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let response = crate::common::CLIENT
            .get(format!("https://vod.afreecatv.com/player/{}", title_no))
            .send()?
            .text()?;
        let title_no = TITLE_NO_MATCHER
            .find(&response)
            .ok_or("nTitleNo missing")?
            .as_str()
            .trim_end_matches(';')[20..]
            .parse()?;
        let station_no = STATION_NO_MATCHER
            .find(&response)
            .ok_or("nStationNo missing")?
            .as_str()
            .trim_end_matches(';')[22..]
            .parse()?;
        let bbs_no = BBS_NO_MATCHER
            .find(&response)
            .ok_or("nBbsNo missing")?
            .as_str()
            .trim_end_matches(';')[18..]
            .parse()?;

        Ok(Self {
            title_no,
            station_no,
            bbs_no,
        })
    }

    fn info_url(&self) -> String {
        format!("https://stbbs.afreecatv.com/api/video/get_video_info.php?nStationNo={}&nBbsNo={}&nTitleNo={}", self.station_no, self.bbs_no, self.title_no)
    }
}

impl std::fmt::Display for Vod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.title_no, self.station_no, self.bbs_no)
    }
}

impl crate::common::Vod for Vod {
    fn comments(&self) -> Box<dyn crate::common::ChatIterator> {
        let xml = crate::common::CLIENT
            .get(self.info_url())
            .send()
            .unwrap()
            .text()
            .unwrap()
            .to_string();
        println!("{}", &xml);
        let key_iter = KEY_MATCHER.find_iter(&xml);
        let duration_iter = DURATION_MATCHER.find_iter(&xml);
        let rows = key_iter
            .zip(duration_iter)
            .map(|n| {
                let key = n.0.as_str()[5..].trim_end_matches('"').to_string();
                let duration = n.1.as_str()[15..].trim_end_matches('"').parse().unwrap();
                Row { key, duration }
            })
            .collect();
        Box::new(ChatIterator { rows })
    }
}

#[derive(Debug)]
struct Row {
    key: String,
    duration: u16,
}

struct ChatIterator {
    rows: std::collections::VecDeque<Row>,
}

impl ChatIterator {
    fn get_segment(key: &str, start_time: u16, time_offset: u16) -> Vec<crate::common::Message> {
        let transcript_url = format!(
            "https://videoimg.afreecatv.com/php/ChatLoadSplit.php?rowKey={}_c&startTime={}",
            key, start_time
        );
        let xml_text = crate::common::CLIENT
            .get(&transcript_url)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let roxml = roxmltree::Document::parse(&xml_text).unwrap();
        let chat = roxml
            .root()
            .descendants()
            .skip_while(|n| n.tag_name().name() != "chat");
        chat.map(|m| m.children().collect::<Vec<roxmltree::Node>>()) 
            .flat_map(|message| -> Option<crate::common::Message> {
                let user = message.get(2)?.text()?;
                let body = message.get(4)?.text()?;
                let timestamp = message.get(6)?.text()?.parse();
                let timestamp = match timestamp {
                    Ok(ts) => ts,
                    Err(_) => return None,
                };
                Some(crate::common::Message {
                    user: user.to_string(),
                    body: body.to_string(),
                    timestamp,
                })
            })
            .collect()
    }

    fn load_chunk(row: Row, time_offset: u16) -> Vec<crate::common::Message> {
        let segment_diff = 300;
        let timings: Vec<u16> =  (0..row.duration)
            .step_by(segment_diff).collect();
        timings.par_iter()
            .map(|t| Self::get_segment(&row.key, *t, time_offset))
            .flatten()
            .collect()
    }
}

impl Iterator for ChatIterator {
    type Item = Vec<crate::common::Message>;
    fn next(&mut self) -> Option<Self::Item> {
        let row = self.rows.pop_front().unwrap();
        let c  =ChatIterator::load_chunk(row, 0);
        dbg!(c);
        None
    }
}

impl crate::common::ChatIterator for ChatIterator {}
