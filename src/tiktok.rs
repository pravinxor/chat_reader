use rayon::prelude::*;

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
}

pub struct ChatIterator {
    id: u64,
    cursor: u64,
}

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.5060.114 Safari/537.36";

impl Iterator for ChatIterator {
    type Item = Vec<crate::common::Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let response: serde_json::Value = crate::common::CLIENT
            .get(format!(
                "https://us.tiktok.com/api/comment/list/?aweme_id={}&count=999999&&cursor={}",
                self.id, self.cursor
            ))
            .header(reqwest::header::REFERER, "https://www.tiktok.com/")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .send()
            .unwrap()
            .json()
            .unwrap();
        self.cursor += 50;
        Some(
            response
                .get("comments")?
                .as_array()?
                .par_iter()
                .flat_map(|comment| -> Option<crate::common::Message> {
                    let user = comment
                        .get("user")?
                        .get("nickname")
                        .unwrap()
                        .as_str()?
                        .to_string();
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
