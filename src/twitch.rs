pub struct Channel {
    username: String,
}

impl Channel {
    pub fn new<S>(username: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            username: username.into(),
        }
    }

    pub fn videos(&self) -> Result<Vec<Vod>, Box<dyn std::error::Error>> {
        let req_json = serde_json::json!([
                                         {
                                             "operationName":"FilterableVideoTower_Videos",
                                             "variables":{
                                                 "limit":100,
                                                 "channelOwnerLogin":self.username,
                                                 "broadcastType":null,
                                                 "videoSort":"TIME",
                                                 "cursor":""
                                             },
                                             "extensions":{
                                                 "persistedQuery":{
                                                     "version":1,
                                                     "sha256Hash":"a937f1d22e269e39a03b509f65a7490f9fc247d7f83d6ac1421523e3b68042cb"
                                                 }
                                             }
                                         }
        ]);

        let response: serde_json::Value = crate::common::CLIENT
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", crate::common::TWITCH_CLIENT_ID)
            .json(&req_json)
            .send()?
            .json()?;
        let vod_json = response
            .get(0)
            .ok_or("Missing idx 0")?
            .get("data")
            .ok_or("Missing data")?
            .get("user")
            .ok_or("Missing user")?
            .get("videos")
            .ok_or("Missing videos; This user may not exist")?
            .get("edges")
            .ok_or("Missing edges")?
            .as_array()
            .ok_or("Unable to convert edges -> array")?;

        let vods: Vec<Vod> = vod_json
            .iter()
            .map(|v| -> Vod {
                let vod = v.get("node").unwrap();
                Vod {
                    title: vod
                        .get("title")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .to_string(),
                    id: vod
                        .get("id")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .parse()
                        .unwrap(),
                    preview_url: vod
                        .get("animatedPreviewURL")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .to_string(),
                }
            })
            .collect();

        Ok(vods)
    }
}

#[derive(Debug)]
pub struct Vod {
    title: String,
    id: u32,
    preview_url: String,
}

impl Vod {
    pub fn new(id: u32) -> Self {
        Self {
            title: String::new(),
            id,
            preview_url: String::new(),
        }
    }
}

impl crate::common::Vod for Vod {
    fn comments(&self) -> Box<dyn crate::common::ChatIterator> {
        Box::new(chat::ChatIterator::new(self.id))
    }
}

impl std::fmt::Display for Vod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.title, self.id)
    }
}

pub mod chat {
    pub struct ChatIterator {
        pub id: u32,
        cursor: Option<String>,
    }

    impl ChatIterator {
        pub fn new(id: u32) -> Self {
            Self {
                id,
                cursor: Some(String::from("")),
            }
        }

        fn get_next(&mut self) -> Result<Vec<crate::common::Message>, Box<dyn std::error::Error>> {
            let request = crate::common::CLIENT
                .get(format!(
                    "https://api.twitch.tv/v5/videos/{}/comments?cursor={}",
                    self.id,
                    self.cursor.as_ref().unwrap()
                ))
                .header("Client-Id", crate::common::TWITCH_CLIENT_ID)
                .send()?;
            let comment_json: serde_json::Value = request.json()?;
            let comments = comment_json
                .get("comments")
                .ok_or("Missing comments; This video ID may not exist")?
                .as_array()
                .ok_or("Unable to convert comments -> array")?;

            let messages = comments
                .iter()
                .map(|comment| -> crate::common::Message {
                    let user = comment
                        .get("commenter")
                        .unwrap()
                        .get("display_name")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .to_string();
                    let message = comment.get("message").unwrap();
                    let body = message
                        .get("body")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .to_string();
                    let colorcode = message.get("user_color");
                    let color = match colorcode {
                        Some(code) => {
                            let code = code.to_string();
                            let code = code.trim_matches('"').trim_start_matches('#');
                            colored::Color::TrueColor {
                                r: u8::from_str_radix(&code[0..2], 16).unwrap(),
                                g: u8::from_str_radix(&code[2..4], 16).unwrap(),
                                b: u8::from_str_radix(&code[4..6], 16).unwrap(),
                            }
                        }
                        None => colored::Color::White,
                    };
                    let timestamp = comment
                        .get("content_offset_seconds")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .parse()
                        .unwrap();
                    crate::common::Message {
                        user,
                        color,
                        body,
                        timestamp,
                    }
                })
                .collect();
            match comment_json.get("_next") {
                Some(next) => self.cursor = Some(next.to_string().trim_matches('"').to_string()),
                None => self.cursor = None,
            }
            Ok(messages)
        }
    }
    impl crate::common::ChatIterator for ChatIterator {}
    impl Iterator for ChatIterator {
        type Item = Vec<crate::common::Message>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.cursor.is_some() {
                match Self::get_next(self) {
                    Ok(messages) => Some(messages),
                    Err(e) => {
                        eprintln!("{}", e);
                        None
                    }
                }
            } else {
                None
            }
        }
    }
}
