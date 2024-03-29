pub struct Vod {
    id: u64,
}

impl Vod {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn comments(&self) -> ChatIterator {
        ChatIterator {
            id: self.id,
            cursor: 0,
        }
    }

    pub fn captions(&self) -> CaptionIterator {
        CaptionIterator { id: self.id }
    }
}

pub struct ChatIterator {
    id: u64,
    cursor: u64,
}

lazy_static::lazy_static! {
    static ref TRANSCRIPT_MATCHER: regex::Regex = regex::Regex::new(r#""eng-U\w","Url":"https:\\u002F\\u002F\w+-webapp.tiktokcdn-\w\w.com\\\w+\\\w+\\u002Fvideo\\u002Ftos\\u002Falisg\\u002Ftos-alisg-pv-\d+\\\w+\\u002F\?"#).unwrap();
}

pub struct CaptionIterator {
    id: u64,
}

impl Iterator for CaptionIterator {
    type Item = Vec<crate::common::Message>;
    fn next(&mut self) -> Option<Self::Item> {
        let response = crate::common::CLIENT
            .get(&format!("https://www.tiktok.com/@tiktok/video/{}", self.id))
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let response = response.as_str();
        println!("{}", response);

        let transcript_key = TRANSCRIPT_MATCHER
            .find(response)?
            .as_str()
            .replace(r#"\u002F"#, "/");
        let transcript_key = transcript_key[16..].trim_end_matches('"');
        println!("{}", transcript_key);
        None
    }
}

impl Iterator for ChatIterator {
    type Item = Vec<crate::common::Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let response: serde_json::Value = crate::common::CLIENT
            .get(&format!(
                "https://us.tiktok.com/api/comment/list/?aweme_id={}&count=50&cursor={}",
                self.id, self.cursor
            ))
            .header(reqwest::header::REFERER, "https://www.tiktok.com/")
            .header(reqwest::header::USER_AGENT, crate::common::USER_AGENT)
            .send()
            .unwrap()
            .json()
            .unwrap();
        self.cursor += 50;
        Some(
            response
                .get("comments")?
                .as_array()?
                .iter()
                .flat_map(|comment| -> Option<crate::common::Message> {
                    let user = Some(
                        comment
                            .get("user")?
                            .get("nickname")
                            .unwrap()
                            .as_str()?
                            .to_string(),
                    );
                    let body = comment.get("text")?.as_str()?.to_string();
                    Some(crate::common::Message {
                        timestamp: None,
                        user,
                        body,
                    })
                })
                .collect(),
        )
    }
}

impl crate::common::ChatIterator for ChatIterator {}
