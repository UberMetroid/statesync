use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
/// Missing documentation.
pub struct WsMessage {
    #[serde(alias = "messageType", alias = "MessageType")]
    /// Missing documentation.
    pub message_type: String,
    #[serde(alias = "data", alias = "Data")]
    /// Missing documentation.
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
/// Missing documentation.
pub struct UserDataChangedInfo {
    #[serde(alias = "userId", alias = "UserId")]
    /// Missing documentation.
    pub user_id: String,
    #[serde(alias = "userDataList", alias = "UserDataList")]
    /// Missing documentation.
    pub user_data_list: Vec<UserDataEntry>,
}

#[derive(Debug, Clone, Deserialize)]
/// Missing documentation.
pub struct UserDataEntry {
    #[serde(alias = "itemId", alias = "ItemId")]
    /// Missing documentation.
    pub item_id: String,
    #[serde(default, alias = "played", alias = "Played")]
    /// Missing documentation.
    pub played: bool,
    #[serde(alias = "playbackPositionTicks", alias = "PlaybackPositionTicks")]
    /// Missing documentation.
    pub playback_position_ticks: Option<i64>,
    #[serde(default, alias = "isFavorite", alias = "IsFavorite")]
    /// Missing documentation.
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
/// Missing documentation.
pub struct SessionInfo {
    #[serde(alias = "id", alias = "Id")]
    /// Missing documentation.
    pub id: String,
    #[serde(alias = "userName", alias = "UserName")]
    /// Missing documentation.
    pub user_name: Option<String>,
    #[serde(alias = "nowPlayingItem", alias = "NowPlayingItem")]
    /// Missing documentation.
    pub now_playing_item: Option<NowPlayingItem>,
    #[serde(alias = "playState", alias = "PlayState")]
    /// Missing documentation.
    pub play_state: Option<PlayState>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
/// Missing documentation.
pub struct NowPlayingItem {
    #[serde(alias = "id", alias = "Id")]
    /// Missing documentation.
    pub id: String,
    #[serde(alias = "name", alias = "Name")]
    /// Missing documentation.
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
/// Missing documentation.
pub struct PlayState {
    #[serde(alias = "positionTicks", alias = "PositionTicks")]
    /// Missing documentation.
    pub position_ticks: Option<i64>,
    #[serde(alias = "isPaused", alias = "IsPaused")]
    /// Missing documentation.
    pub is_paused: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
/// Missing documentation.
pub struct PlayedItem {
    #[serde(alias = "Id", alias = "id")]
    /// Missing documentation.
    pub id: String,
    #[serde(default, alias = "Played", alias = "played")]
    /// Missing documentation.
    pub played: bool,
    #[serde(
        default,
        alias = "PlaybackPositionTicks",
        alias = "playbackPositionTicks"
    )]
    /// Missing documentation.
    pub playback_position_ticks: Option<i64>,
    #[serde(default, alias = "IsFavorite", alias = "isFavorite")]
    /// Missing documentation.
    pub is_favorite: Option<bool>,
    #[serde(default, alias = "LastPlayedDate", alias = "lastPlayedDate")]
    /// Missing documentation.
    pub last_played_date: Option<String>,
    #[serde(default)]
    /// Missing documentation.
    pub imdb_id: Option<String>,
    #[serde(default)]
    /// Missing documentation.
    pub tmdb_id: Option<String>,
}
