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

    pub async fn videos(&self) -> Result<Vec<Vod>, Box<dyn std::error::Error>> {
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
            .send()
            .await?
            .json()
            .await?;
        let vod_json = response
            .get(0)
            .ok_or("Missing idx 0")?
            .get("data")
            .ok_or("Missing data")?
            .get("user")
            .ok_or("Missing user")?
            .get("videos")
            .ok_or("Missing videos")?
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

    pub fn comments(&self) -> chat::ChatIterator {
        chat::ChatIterator::new(self.id)
    }
}

pub mod chat {
    pub struct ChatIterator {
        id: u32,
        cursor: Option<String>,
    }

    #[derive(Debug)]
    pub struct Message {
        user: String,
        color: u16,
        body: String,
        timestamp: f64,
    }

    impl ChatIterator {
        pub fn new(id: u32) -> Self {
            Self {
                id,
                cursor: Some(String::from("")),
            }
        }

        async fn get_next(&mut self) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
            let comment_json: serde_json::Value = crate::common::CLIENT
                .get(format!(
                    "https://api.twitch.tv/v5/videos/{}/comments?cursor={}",
                    self.id,
                    self.cursor.as_ref().unwrap()
                ))
                .header("Client-Id", crate::common::TWITCH_CLIENT_ID)
                .send()
                .await?
                .json()
                .await?;
            dbg!(self.cursor.as_ref().unwrap());
            let comments = comment_json
                .get("comments")
                .ok_or("Missing comments")?
                .as_array()
                .ok_or("Unable to convert comments -> array")?;

            let messages = comments
                .iter()
                .map(|comment| -> Message {
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
                    let timestamp = comment
                        .get("content_offset_seconds")
                        .unwrap()
                        .to_string()
                        .trim_matches('"')
                        .parse()
                        .unwrap();
                    Message {
                        user,
                        color: 0,
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
    impl Iterator for ChatIterator {
        type Item = Vec<Message>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.cursor.is_some() {
                match futures::executor::block_on(Self::get_next(self)) {
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
