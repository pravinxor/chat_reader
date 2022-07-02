use rayon::prelude::*;

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
        
        let vods: Vec<Vod> = vod_json.par_iter().map(|v| -> Vod {
            let vod = v.get("node").unwrap();
            Vod {
                title: vod
                    .get("title")
                    .unwrap()
                    .to_string()
                    .trim_matches('"')
                    .to_string(),
                id: vod.get("id")
                    .unwrap()
                    .to_string()
                    .trim_matches('"')
                    .to_string()
                    .parse()
                    .unwrap(),
                preview_url: vod
                    .get("animatedPreviewURL")
                    .unwrap()
                    .to_string()
                    .trim_matches('"')
                    .to_string()
            }
        }).collect();

        Ok(vods)
    }
}

#[derive(Debug)]
pub struct Vod {
    title: String,
    id: u32,
    preview_url: String,
}
