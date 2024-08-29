use crate::helper::error_management::error_dispatch;
use rand::Rng;
use rusty_ytdl::search::{SearchOptions, SearchType};
use rusty_ytdl::{
    search::{SearchResult, YouTube},
    Video,
};
use rusty_ytdl::{RequestOptions, VideoOptions};
use rusty_ytdl::{VideoError, VideoQuality, VideoSearchOptions};
use serenity::async_trait;
use songbird::input::{AudioStream, AudioStreamError, AuxMetadata, Compose, Input};
use std::fs;
use std::path::Path;
use std::time::Duration;
use symphonia::core::io::{
    MediaSource, MediaSourceStream, MediaSourceStreamOptions, ReadOnlySource,
};
use tracing::trace;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum UrlType {
    Video,
    Playlist,
}

impl From<UrlType> for SearchType {
    fn from(val: UrlType) -> Self {
        match val {
            UrlType::Video => SearchType::Video,
            UrlType::Playlist => SearchType::Playlist,
        }
    }
}

impl From<SearchType> for UrlType {
    fn from(val: SearchType) -> Self {
        match val {
            SearchType::Video => UrlType::Video,
            SearchType::Playlist => UrlType::Playlist,
            _ => UrlType::Video,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RustyYoutubeSearch {
    pub rusty_ytdl: YouTube,
    pub metadata: Option<AuxMetadata>,
    pub url: Option<String>,
    pub search: Option<String>,
    pub video: Option<Video>,
    pub url_type: UrlType,
}

impl RustyYoutubeSearch {
    pub fn new_from_url(url: String) -> Result<Self, VideoError> {
        Ok(Self {
            rusty_ytdl: YouTube::new()?,
            metadata: None,
            url: Some(url),
            search: None,
            video: None,
            url_type: UrlType::Video,
        })
    }

    pub fn new_from_search(query: String) -> Result<Self, VideoError> {
        Ok(Self {
            rusty_ytdl: YouTube::new()?,
            metadata: None,
            url: None,
            search: Some(query),
            video: None,
            url_type: UrlType::Video,
        })
    }
}

impl From<RustyYoutubeSearch> for Input {
    fn from(val: RustyYoutubeSearch) -> Self {
        Input::Lazy(Box::new(val))
    }
}

#[async_trait]
impl Compose for RustyYoutubeSearch {
    fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        Err(AudioStreamError::Unsupported)
    }

    async fn create_async(
        &mut self,
    ) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        // check if valid_proxy.txt exists
        let proxy_path = Path::new("valid_proxy.txt");
        let proxy = proxy_path.exists();

        if self.metadata.is_none() {
            self.aux_metadata().await?;
        }

        let url = self.url.clone().unwrap_or_default();

        let request_options = if proxy {
            trace!("Using proxy");
            let proxy = fs::read_to_string(proxy_path).map_err(|e| {
                AudioStreamError::Fail(
                    error_dispatch::Error::Audio(format!("Failed to read proxy file: {e:?}"))
                        .into(),
                )
            })?;
            let proxy = proxy.split("\n").collect::<Vec<&str>>();
            let mut rng = rand::thread_rng();
            let n = rng.gen_range(0..proxy.len());
            let proxy = proxy[n];
            let proxy = reqwest::Proxy::all(proxy).map_err(|e| {
                AudioStreamError::Fail(
                    error_dispatch::Error::Audio(format!("Failed to create proxy: {e:?}")).into(),
                )
            })?;
            RequestOptions {
                proxy: Some(proxy),
                ..Default::default()
            }
        } else {
            Default::default()
        };

        let video = Video::new_with_options(
            url.clone(),
            VideoOptions {
                quality: VideoQuality::HighestAudio,
                filter: VideoSearchOptions::Audio,
                download_options: Default::default(),
                request_options,
            },
        )
        .map_err(|e| {
            AudioStreamError::Fail(
                error_dispatch::Error::Audio(format!("Failed to create stream: {e:?}")).into(),
            )
        })?;

        let tempdir = tempfile::tempdir().map_err(|e| {
            AudioStreamError::Fail(
                error_dispatch::Error::Audio(format!("Failed to create tempdir: {e:?}")).into(),
            )
        })?;
        trace!("Downloading video to {:?}", tempdir.path());
        let uuid = Uuid::new_v4();
        let path = tempdir.path().join(format!("{}.mp4", uuid));
        video.download(&path).await.map_err(|e| {
            AudioStreamError::Fail(
                error_dispatch::Error::Audio(format!("Failed to download video: {e:?}")).into(),
            )
        })?;
        trace!("Downloaded video");

        let file = fs::File::open(&path).map_err(|e| {
            AudioStreamError::Fail(
                error_dispatch::Error::Audio(format!("Failed to open file: {e:?}")).into(),
            )
        })?;

        let ros = ReadOnlySource::new(file);
        let source = MediaSourceStream::new(Box::new(ros), MediaSourceStreamOptions::default());

        Ok(AudioStream {
            input: Box::new(source),
            hint: None,
        })
    }

    fn should_create_async(&self) -> bool {
        true
    }

    async fn aux_metadata(&mut self) -> Result<AuxMetadata, AudioStreamError> {
        if let Some(meta) = self.metadata.as_ref() {
            return Ok(meta.clone());
        }
        let data = self
            .url
            .clone()
            .unwrap_or(self.search.clone().unwrap_or_default());
        let res: SearchResult = self
            .rusty_ytdl
            .search_one(
                data,
                Some(&SearchOptions {
                    limit: 1,
                    search_type: self.url_type.clone().into(),
                    safe_search: false,
                }),
            )
            .await
            .map_err(|e| AudioStreamError::Fail(e.into()))?
            .ok_or(AudioStreamError::Fail(Box::from(String::from(
                "No video found",
            ))))?;

        let mut metadata = AuxMetadata::default();
        match res.clone() {
            SearchResult::Video(video) => {
                self.url_type = UrlType::Video;

                metadata.track = Some(video.title.clone());
                metadata.artist = None;
                metadata.album = None;
                metadata.date = video.uploaded_at.clone();

                metadata.channels = Some(2);
                metadata.channel = Some(video.channel.name);
                metadata.duration = Some(Duration::from_millis(video.duration));
                metadata.sample_rate = Some(48000);
                metadata.source_url = Some(video.url);
                metadata.title = Some(video.title);
                metadata.thumbnail = Some(video.thumbnails.first().unwrap().url.clone());
            }
            SearchResult::Playlist(playlist) => {
                self.url_type = UrlType::Playlist;
                metadata.title = Some(playlist.name);
                metadata.source_url = Some(playlist.url);
                metadata.duration = None;
                metadata.thumbnail = Some(playlist.thumbnails.first().unwrap().url.clone());
            }
            _ => {}
        };
        self.search = Some(metadata.title.clone().unwrap_or_default());
        self.url = Some(metadata.source_url.clone().unwrap_or_default());

        self.metadata = Some(metadata.clone());
        Ok(metadata)
    }
}
