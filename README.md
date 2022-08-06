# Chat Reader
A fast tool for sifting through transcripts across platforms, with an assortment of other features as well!

#### Print the entire chat transcript
`./chat_reader twitch vod "1234567890"`

#### Now filter for a few words
`./chat_reader -f "nerd|meme" twitch vod "1234567890"`

#### Now do it for every video in a channel
`./chat_reader -f "nerd|meme" twitch channel --vods "twitch"`

#### You can also do this for every channel in a game directory
`./chat_reader -f "nerd|meme" twitch directory --vods "Just Chatting"`

#### In addition to looking through the chats, you can also look through clips
`./chat_reader -f "nerd|meme" twitch directory --vods --clips "Just Chatting"`

#### You can also try to recover VODs from a Twitch channel, if they've been removed recently
`./chat_reader twitch channel --recover "twitch"`

### There is currently support for Twitch & AfreecaTV along with basic functionality for TikTok, with more robust support coming later on
