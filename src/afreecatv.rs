use rayon::prelude::*;

pub struct Channel {
    name: String,
}

impl Channel {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }

    fn get_page(&self, num: u64) -> Result<Vec<Vod>, Box<dyn std::error::Error>> {
        let vods_json: serde_json::Value = crate::common::CLIENT
            .get(format!(
                "https://bjapi.afreecatv.com/api/{}/vods/all?per_page=60&page={}",
                self.name, num
            ))
            .send()?
            .json()?;
        let data = vods_json
            .get("data")
            .ok_or("Missing data")?
            .as_array()
            .ok_or("Unable to convert data -> array")?;

        Ok(data
            .iter()
            .flat_map(|v| -> Option<Vod> {
                let title_name = v.get("title_name")?.as_str()?.to_string();
                let title_no = v.get("title_no")?.as_u64()? as u32;
                let station_no = v.get("station_no")?.as_u64()? as u32;
                let bbs_no = v.get("bbs_no")?.as_u64()? as u32;

                Some(Vod {
                    title_name: Some(title_name),
                    title_no,
                    station_no,
                    bbs_no,
                })
            })
            .collect())
    }

    pub fn videos(&self) -> Result<Vec<Vod>, Box<dyn std::error::Error>> {
        let info_json: serde_json::Value = crate::common::CLIENT
            .get(format!(
                "https://bjapi.afreecatv.com/api/{}/vods/all?per_page=60",
                self.name
            ))
            .send()?
            .json()?;
        let last_page = info_json
            .get("meta")
            .ok_or("Missing meta")?
            .get("last_page")
            .ok_or("Missing last_page")?
            .as_u64()
            .ok_or("Unable to convert last_page -> u64")?;
        let v = (1..=last_page)
            .into_par_iter()
            .flat_map(|n| self.get_page(n))
            .flatten()
            .collect();
        Ok(v)
    }
}

pub struct Vod {
    title_name: Option<String>,
    title_no: u32,
    station_no: u32,
    bbs_no: u32,
}

const DUMMY_COOKIE: &str = "PdboxTicket=.A32.7bbT56vyHM9fKZk.SCwwbeEYGl-_RK8offHEfHRYug37IvxHp0iHV0ZjIqUgEYDviDxevQx01PU6-AIlExXpKM5FEovtC9uP5EjNQPDwZy2I1EjK9l8WItbBrj5hT7jYYNI34878csX4CiR0cVbPPGjlXxk3U_b3F6jxpL7wjHq1-Bn7H9-CeE-OCrOn1b_4A-pWHT-hESimjmpn4vuuyKPahezPgzUYwUI6aEfA5tDmg4a5QbHbi6i6bzGHl-QNgOU4fBpptWbAWhP1ozM-fgpezDX48KiJhMUKRnxFxqVRL_gRVUUDKdKj6gWMi1eSQ_tZGZ5WjTaU6HedqKCoTu3uyFhXObO9hBgUA7UDNanINf4t4SqfDissg9Rg0nsnM8IiTLF7Jpk_XsBuna1K59t_92Ogf3AP7FrCw0rgX43RWBmM4g1sNbo8WPV17zbUg-3F6GRcGZDPqqPHIh6Wz5Aralleoo0ZkFl73uVgBkKJyuw69rouH_k3MgbHRfdUMoh-i6pLcgDG63zhCp3KF0lbzXV4aaLqj7ZIoSpKkEJJ2Q-nS5JaQrFXfAuf_iB5mviN57PDdulJF3cA; PdboxUser=uid%3Dtbfftemp1873%26uno%3D43888847%26age%3D46%26sex%3DA%26A%3DAAG%26B%3DBAEG%26unick%26apply_date%3D1625675007%26name_chk%3D1%26sess_adult_chk%3D1%26broad_name_chk%3D1%26change_password%3D1%26chnnl_cd%3D1%26chnnl_name_chk%3D1";

lazy_static::lazy_static! {
    static ref TITLE_NO_MATCHER: regex::Regex = regex::Regex::new(r#"document\.nTitleNo = [0-9]+;"#).unwrap();
    static ref STATION_NO_MATCHER: regex::Regex = regex::Regex::new(r#"document\.nStationNo = [0-9]+;"#).unwrap();
    static ref BBS_NO_MATCHER: regex::Regex = regex::Regex::new(r#"document\.nBbsNo = [0-9]+;"#).unwrap();

    static ref KEY_MATCHER: regex::Regex = regex::Regex::new(r#"key="([0-9]|[A-Z]|_)+""#).unwrap();
    static ref DURATION_MATCHER: regex::Regex = regex::Regex::new(r#"file duration="[0-9]+""#).unwrap();
}

impl std::fmt::Display for Vod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(title_name) = &self.title_name {
            write!(f, "{} {}", title_name, self.title_no)
        } else {
            write!(f, "{}", self.title_no)
        }
    }
}

impl Vod {
    pub fn new(title_no: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let response = crate::common::CLIENT
            .get(format!("https://vod.afreecatv.com/player/{}", title_no))
            .header(reqwest::header::COOKIE, DUMMY_COOKIE)
            .send()?
            .text()?;
        let response = response.as_str();
        let title_no = TITLE_NO_MATCHER
            .find(response)
            .ok_or("nTitleNo missing")?
            .as_str()
            .trim_end_matches(';')[20..]
            .parse()?;
        let station_no = STATION_NO_MATCHER
            .find(response)
            .ok_or("nStationNo missing")?
            .as_str()
            .trim_end_matches(';')[22..]
            .parse()?;
        let bbs_no = BBS_NO_MATCHER
            .find(response)
            .ok_or("nBbsNo missing")?
            .as_str()
            .trim_end_matches(';')[18..]
            .parse()?;

        Ok(Self {
            title_name: None,
            title_no,
            station_no,
            bbs_no,
        })
    }

    fn info_url(&self) -> String {
        format!("https://stbbs.afreecatv.com/api/video/get_video_info.php?nStationNo={}&nBbsNo={}&nTitleNo={}", self.station_no, self.bbs_no, self.title_no)
    }
}

impl crate::common::Vod for Vod {
    fn comments(&self) -> Box<dyn crate::common::ChatIterator> {
        let xml = crate::common::CLIENT
            .get(&self.info_url())
            .header(reqwest::header::COOKIE, DUMMY_COOKIE)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let xml = xml.as_str();
        let key_iter = KEY_MATCHER.find_iter(xml);
        let duration_iter = DURATION_MATCHER.find_iter(xml);
        let rows = key_iter
            .zip(duration_iter)
            .map(|n| {
                let key = n.0.as_str()[5..].trim_end_matches('"').to_string();
                let duration = n.1.as_str()[15..].trim_end_matches('"').parse().unwrap();
                Row { key, duration }
            })
            .collect();
        Box::new(ChatIterator {
            rows,
            current_offset: 0,
        })
    }
}

#[derive(Debug)]
struct Row {
    key: String,
    duration: u16,
}

struct ChatIterator {
    rows: std::collections::VecDeque<Row>,
    current_offset: u16,
}

impl ChatIterator {
    fn get_segment(
        key: &str,
        start_time: u16,
        time_offset: u16,
    ) -> Result<Vec<crate::common::Message>, Box<dyn std::error::Error>> {
        let transcript_url = format!(
            "https://videoimg.afreecatv.com/php/ChatLoadSplit.php?rowKey={}_c&startTime={}",
            key, start_time
        );
        let xml_text = crate::common::CLIENT.get(transcript_url).send()?.text()?;
        let xml_text = xml_text.as_str();
        let roxml = roxmltree::Document::parse(xml_text)?;
        let chat = roxml
            .root()
            .descendants()
            .skip_while(|n| n.tag_name().name() != "chat");
        Ok(chat
            .map(|m| m.children().collect::<Vec<roxmltree::Node>>())
            .flat_map(|message| -> Option<crate::common::Message> {
                let user = message.get(2)?.text()?;
                let body = message.get(4)?.text()?;
                let timestamp = message.get(6)?.text()?.parse();
                let timestamp: f64 = match timestamp {
                    Ok(ts) => ts,
                    Err(_) => return None,
                };
                Some(crate::common::Message {
                    user: Some(user.to_string()),
                    body: body.to_string(),
                    timestamp: Some(timestamp + time_offset as f64),
                })
            })
            .collect())
    }

    fn load_chunk(row: Row, time_offset: u16) -> Vec<crate::common::Message> {
        let segment_diff = 300;
        let timings: Vec<u16> = (0..row.duration).step_by(segment_diff).collect();
        timings
            .par_iter()
            .flat_map(|t| Self::get_segment(&row.key, *t, time_offset))
            .flatten()
            .collect()
    }
}

impl Iterator for ChatIterator {
    type Item = Vec<crate::common::Message>;
    fn next(&mut self) -> Option<Self::Item> {
        let row = self.rows.pop_front()?;
        let duration = row.duration;
        let item = ChatIterator::load_chunk(row, self.current_offset);
        self.current_offset += duration;
        Some(item)
    }
}

impl crate::common::ChatIterator for ChatIterator {}
