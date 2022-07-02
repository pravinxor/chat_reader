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

    pub async fn videos(&self) -> () {
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

        let reponse: serde_json::Value = crate::common::CLIENT
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", crate::common::TWITCH_CLIENT_ID)
            .json(&req_json)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        dbg!(reponse);
    }
}
