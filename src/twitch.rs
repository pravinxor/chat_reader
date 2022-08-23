use rayon::prelude::*;

const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
const GQL: &str = "https://gql.twitch.tv/gql";

#[derive(PartialEq)]
pub enum Recency {
    AllTime,
    LastMonth,
    LastWeek,
    LastDay,
}

impl Recency {
    fn as_str(&self) -> &'static str {
        match self {
            Recency::AllTime => "ALL_TIME",
            Recency::LastMonth => "LAST_MONTH",
            Recency::LastWeek => "LAST_WEEK",
            Recency::LastDay => "LAST_DAY",
        }
    }
}

impl std::fmt::Display for Recency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Recency {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ALL_TIME" => Ok(Recency::AllTime),
            "LAST_MONTH" => Ok(Recency::LastMonth),
            "LAST_WEEK" => Ok(Recency::LastWeek),
            "LAST_DAY" => Ok(Recency::LastDay),
            _ => Err(r#"Expected: ["ALL_TIME", "LAST_MONTH", "LAST_WEEK", "LAST_DAY"]"#),
        }
    }
}

#[derive(Debug)]
pub struct Directory {
    name: String,
}

impl Directory {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }

    pub fn channels(&self) -> DirectoryIterator {
        DirectoryIterator {
            name: &self.name,
            cursor: String::from(""),
        }
    }
    pub fn clips(&self, recency: Recency) -> DirectoryClipIterator {
        DirectoryClipIterator {
            name: &self.name,
            recency,
            cursor: Some(String::from("")),
        }
    }
}

pub struct DirectoryClipIterator<'a> {
    name: &'a str,
    recency: Recency,
    cursor: Option<String>,
}

impl Iterator for DirectoryClipIterator<'_> {
    type Item = Vec<crate::common::Message>;
    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.as_ref()?;

        let req_json = serde_json::json!([{
            "operationName": "ClipsCards__Game",
            "variables": {
                "gameName": self.name,
                "limit": 20,
                "cursor": self.cursor.as_ref().unwrap(),
                "criteria": {
                    "languages": [],
                    "filter": self.recency.as_str()
                }
            },
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "0d8d0eba9fc7ef77de54a7d933998e21ad7a1274c867ec565ac14ffdce77b1f9"
                }
            }
        }]);
        self.cursor = None;

        let edges;
        loop {
            let response: serde_json::Value = crate::common::CLIENT
                .post(GQL)
                .header("Client-Id", CLIENT_ID)
                .header("X-Device-Id", "1UTTXkkDGQnD17zO8HvZ2mFiFONpG1ft")
                .json(&req_json)
                .send()
                .unwrap()
                .json()
                .unwrap();

            let clips = response.get(0)?.get("data")?.get("game")?.get("clips")?;
            if clips.is_null() {
                continue;
            }
            edges = clips.get("edges")?.as_array()?.to_owned();
            break;
        }

        self.cursor = edges
            .iter()
            .map(|edge| edge.get("cursor")?.as_str())
            .last()
            .map(|cursor| cursor.unwrap().to_owned());

        Some(
            edges
                .iter()
                .flat_map(|edge| edge.get("node"))
                .flat_map(|node| -> Option<crate::common::Message> {
                    let title = node.get("title")?.as_str()?;
                    let slug = node.get("slug")?.as_str()?;
                    let user = node.get("broadcaster")?.get("displayName")?.as_str()?;

                    let body = format!("[{}] {}", title, slug);
                    Some(crate::common::Message {
                        body,
                        timestamp: None,
                        user: Some(user.to_owned()),
                    })
                })
                .collect(),
        )
    }
}

pub struct DirectoryIterator<'a> {
    name: &'a str,
    cursor: String,
}

impl Iterator for DirectoryIterator<'_> {
    type Item = Vec<Channel>;
    fn next(&mut self) -> Option<Self::Item> {
        let req_json = serde_json::json!([{
            "operationName": "DirectoryPage_Game",
            "variables": {
                "imageWidth": 50,
                "name": self.name,
                "options": {
                    "includeRestricted": [
                        "SUB_ONLY_LIVE"
                    ],
                    "sort": "RELEVANCE",
                    "recommendationsContext": {
                        "platform": "web"
                    },
                    "requestID": "JIRA-VXP-2397",
                    "freeformTags": null,
                    "tags":[]
                },
                "freeformTagsEnabled": false,
                "sortTypeIsRecency": false,
                "limit": 30,
                "cursor": self.cursor
            },
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "749035333f1837aca1c5bae468a11c39604a91c9206895aa90b4657ab6213c24"
                }
            }
        }]);
        let response: serde_json::Value = crate::common::CLIENT
            .post(GQL)
            .header("Client-Id", CLIENT_ID)
            .header("X-Device-Id", "1UTTXkkDGQnD17zO8HvZ2mFiFONpG1ft")
            .json(&req_json)
            .send()
            .unwrap()
            .json()
            .unwrap();

        let channels = response
            .get(0)?
            .get("data")?
            .get("game")?
            .get("streams")?
            .get("edges")?
            .as_array()?;
        let out = channels
            .iter()
            .flat_map(|edge| edge.get("node"))
            .flat_map(|node| node.get("broadcaster"))
            .flat_map(|broadcaster| -> Option<Channel> {
                let username = broadcaster.get("login")?.as_str()?;
                Some(Channel {
                    username: username.into(),
                })
            })
            .collect();
        if let Some(cursor) = channels.get(25)?.get("cursor") {
            self.cursor = cursor.as_str()?.to_string();
            Some(out)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Tag;

impl Tag {
    pub fn channels(tags: &[String]) -> TagIterator {
        TagIterator {
            tags: tags.to_vec(),
            cursor: Some(String::from("")),
        }
    }
}

pub struct TagIterator {
    tags: Vec<String>,
    cursor: Option<String>,
}

impl Iterator for TagIterator {
    type Item = Vec<Channel>;
    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.as_ref()?;
        let req_json = serde_json::json!([
        {
            "operationName": "BrowsePage_Popular",
            "variables": {
                "imageWidth": 50,
                "limit": 30,
                "platformType": "all",
                "options": {
                    "sort": "RELEVANCE",
                    "freeformTags": self.tags,
                    "recommendationsContext": {
                        "platform": "web"
                    },
                    "requestID": "JIRA-VXP-2397"
                },
                "sortTypeIsRecency": false,
                "freeformTagsEnabled": true,
                "cursor": self.cursor,
            },
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "267d2d2a64e0a0d6206c039ea9948d14a9b300a927d52b2efc52d2486ff0ec65"
                }
            }
        }]);
        let request: serde_json::Value = crate::common::CLIENT
            .post(GQL)
            .header("Client-Id", CLIENT_ID)
            .header("X-Device-Id", "1UTTXkkDGQnD17zO8HvZ2mFiFONpG1ft")
            .json(&req_json)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let streamlist = request
            .get(0)?
            .get("data")?
            .get("streams")?
            .get("edges")?
            .as_array()?;
        let out = streamlist
            .iter()
            .flat_map(|e| -> Option<Channel> {
                let username = e.get("node")?.get("broadcaster")?.get("login")?.as_str()?;
                Some(Channel::new(username))
            })
            .collect();
        self.cursor = if let Some(stream) = streamlist.get(25) {
            if let Some(cursor) = stream.get("cursor") {
                cursor.as_str().map(|cursor| cursor.to_owned())
            } else {
                None
            }
        } else {
            None
        };
        Some(out)
    }
}

#[derive(Debug)]
pub struct Channel {
    pub username: String,
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

    pub fn clips(&self) -> self::clips::ClipIterator {
        self::clips::ClipIterator {
            username: &self.username,
            cursor: Some(String::from("")),
        }
    }

    pub fn videos(&self) -> Result<Vec<Vod>, Box<dyn std::error::Error>> {
        let req_json = serde_json::json!([
                                         {
                                             "operationName": "FilterableVideoTower_Videos",
                                             "variables": {
                                                 "limit": 100,
                                                 "channelOwnerLogin": self.username,
                                                 "broadcastType": null,
                                                 "videoSort": "TIME",
                                                 "cursor": ""
                                             },
                                             "extensions": {
                                                 "persistedQuery": {
                                                     "version": 1,
                                                     "sha256Hash": "a937f1d22e269e39a03b509f65a7490f9fc247d7f83d6ac1421523e3b68042cb"
                                                 }
                                             }
                                         }
        ]);

        let response: serde_json::Value = crate::common::CLIENT
            .post(GQL)
            .header("Client-Id", CLIENT_ID)
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
            .par_iter()
            .flat_map(|v| -> Option<Vod> {
                let vod = v.get("node").unwrap();
                let title = vod.get("title")?.as_str()?.to_string();
                let id = vod.get("id").unwrap().as_str()?.parse().unwrap();
                let m3u8 = Vod::m3u8(id, vod.get("animatedPreviewURL")?.as_str()?).unwrap();
                Some(Vod { title, id, m3u8 })
            })
            .collect();
        Ok(vods)
    }
}

pub mod clips {
    pub struct ClipIterator<'a> {
        pub username: &'a str,
        pub cursor: Option<String>,
    }

    impl ClipIterator<'_> {
        fn get_next(&mut self) -> Result<Vec<crate::common::Message>, Box<dyn std::error::Error>> {
            let req_json = serde_json::json!([
                                             {
                                                 "operationName": "ClipsCards__User",
                                                 "variables": {
                                                 "login": self.username,
                                                 "limit": 100,
                                                 "criteria": {
                                                     "filter": super::Recency::AllTime.as_str()
                                                 },
                                                 "cursor": self.cursor,
                                                 },
                                                 "extensions": {
                                                     "persistedQuery": {
                                                     "version": 1,
                                                     "sha256Hash": "b73ad2bfaecfd30a9e6c28fada15bd97032c83ec77a0440766a56fe0bd632777"
                                                 }
                                             }
                                         }
            ]);

            let response: serde_json::Value = crate::common::CLIENT
                .post(super::GQL)
                .header("Client-Id", super::CLIENT_ID)
                .json(&req_json)
                .send()?
                .json()?;
            let clips = response
                .get(0)
                .ok_or("Missing idx 0")?
                .get("data")
                .ok_or("Missing data")?
                .get("user")
                .ok_or("Missing user")?
                .get("clips")
                .ok_or("Missing clips")?
                .get("edges")
                .ok_or("Missing edges")?
                .as_array()
                .ok_or("Unable to convert clips -> array")?;
            self.cursor = clips
                .iter()
                .flat_map(|e| -> Option<String> { Some(e.get("cursor")?.as_str()?.into()) })
                .next();
            Ok(clips
                .iter()
                .flat_map(|e| e.get("node"))
                .flat_map(|node| -> Option<crate::common::Message> {
                    let user = node.get("curator")?.get("displayName")?.as_str()?;
                    let slug = node.get("slug")?.as_str()?;
                    let title = node.get("title")?.as_str()?;

                    let body = format!("[{}] {}", title, slug);
                    Some(crate::common::Message {
                        user: Some(user.into()),
                        timestamp: None,
                        body,
                    })
                })
                .collect())
        }
    }
    impl Iterator for ClipIterator<'_> {
        type Item = Vec<crate::common::Message>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.cursor.is_some() {
                match self.get_next() {
                    Ok(next) => Some(next),
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

#[derive(Debug)]
pub struct Vod {
    title: String,
    id: u32,
    m3u8: String,
}

impl Vod {
    pub fn new(id: u32) -> Self {
        Self {
            title: String::new(),
            id,
            m3u8: String::new(),
        }
    }

    fn m3u8(id: u32, preview_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        if preview_url.is_empty() {
            return Ok(format!("https://twitch.tv/videos/{}", id));
        }

        let chunked_index = preview_url
            .find("storyboards")
            .ok_or("Could not find storboards")?;
        let domain_url = format!("{}chunked/", &preview_url[..chunked_index]);
        let req_json = serde_json::json!([
                                         {
                                             "operationName": "VideoMetadata",
                                             "variables": {
                                                 "channelLogin": "",
                                                 "videoID": format!(r#"{}"#, id),
                                             },
                                             "extensions": {
                                                 "persistedQuery": {
                                                     "version": 1,
                                                     "sha256Hash": "226edb3e692509f727fd56821f5653c05740242c82b0388883e0c0e75dcbf687"
                                                 }
                                             }
                                         }
        ]);

        let reponse = crate::common::CLIENT
            .post(GQL)
            .header("Client-Id", CLIENT_ID)
            .json(&req_json)
            .send()?;
        let metadata_json: serde_json::Value = reponse.json()?;
        let vod_type = metadata_json
            .get(0)
            .ok_or("Missing idx 0")?
            .get("data")
            .ok_or("Missing data")?
            .get("video")
            .ok_or("Missing video")?
            .get("broadcastType")
            .ok_or("Missing broadcast type")?
            .as_str()
            .ok_or("Could not convert to string")?;
        Ok(match vod_type {
            "HIGHLIGHT" => format!("{}highlight-{}.m3u8", domain_url, id),
            "ARCHIVE" => format!("{}index-dvr.m3u8", domain_url),
            _ => format!("https://twitch.tv/videos/{}", id),
        })
    }
}

impl crate::common::Vod for Vod {
    fn comments(&self) -> Box<dyn crate::common::ChatIterator> {
        Box::new(chat::ChatIterator::new(self.id))
    }
}

impl std::fmt::Display for Vod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\n{}", self.title, self.id, self.m3u8)
    }
}

mod chat {
    use colored::Colorize;

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
                    self.cursor.as_ref().ok_or("Cursor is None")?
                ))
                .header("Client-Id", crate::twitch::CLIENT_ID)
                .send()?;
            let comment_json: serde_json::Value = request.json()?;
            let comments = comment_json
                .get("comments")
                .ok_or("Missing comments; This video ID may not exist")?
                .as_array()
                .ok_or("Unable to convert comments -> array")?;

            let messages = comments
                .iter()
                .filter_map(|comment| -> Option<crate::common::Message> {
                    let mut user: Option<String> =
                        Some(comment.get("commenter")?.get("name")?.as_str()?.into());
                    let message = comment.get("message")?;
                    let body = message.get("body")?.as_str()?.into();
                    let colorcode = message.get("user_color");
                    let color = match colorcode {
                        Some(code) => {
                            let code = code.as_str()?.trim_start_matches('#');
                            Some(colored::Color::TrueColor {
                                r: u8::from_str_radix(&code[0..2], 16).unwrap(),
                                g: u8::from_str_radix(&code[2..4], 16).unwrap(),
                                b: u8::from_str_radix(&code[4..6], 16).unwrap(),
                            })
                        }
                        None => None,
                    };
                    if let Some(color) = color {
                        user = Some(user.unwrap().color(color).to_string());
                    }
                    let timestamp = comment.get("content_offset_seconds")?.as_f64()?;

                    Some(crate::common::Message {
                        user,
                        body,
                        timestamp: Some(timestamp),
                    })
                })
                .collect();
            match comment_json.get("_next") {
                Some(next) => self.cursor = Some(next.as_str().unwrap().into()),
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
