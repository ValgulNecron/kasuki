use crate::helper::error_management::error_dispatch;
use prost::bytes::{Buf, BytesMut};
use rusty_ytdl::search::{SearchOptions, SearchType};
use rusty_ytdl::stream::Stream;
use rusty_ytdl::VideoOptions;
use rusty_ytdl::{
    search::{SearchResult, YouTube},
    Video,
};
use rusty_ytdl::{VideoError, VideoQuality, VideoSearchOptions};
use serenity::async_trait;
use songbird::input::core::io::MediaSource;
use songbird::input::{AudioStream, AudioStreamError, AuxMetadata, Compose, Input};
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

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

impl Into<UrlType> for SearchType {
    fn into(self) -> UrlType {
        match self {
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
        if self.metadata.is_none() {
            self.aux_metadata().await?;
        }

        let url = self.url.clone().unwrap_or_default();

        Video::new_with_options(
            url.clone(),
            VideoOptions {
                quality: VideoQuality::HighestAudio,
                filter: VideoSearchOptions::Audio,
                download_options: Default::default(),
                request_options: Default::default(),
            },
        )
        .map_err(|e| {
            AudioStreamError::Fail(
                error_dispatch::Error::Audio(format!("Failed to create stream: {e:?}")).into(),
            )
        })?
        .stream()
        .await
        .map(|input| {
            // let stream = AsyncAdapterStream::new(input, 64 * 1024);
            let stream = Box::into_pin(input).into_media_source();

            AudioStream {
                input: Box::new(stream) as Box<dyn MediaSource>,
                hint: None,
            }
        })
        .map_err(|e| AudioStreamError::Fail(e.into()))
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

pub trait StreamExt {
    fn into_media_source(self: Pin<Box<Self>>) -> MediaSourceStream;
}

impl StreamExt for dyn Stream + Sync + Send {
    fn into_media_source(self: Pin<Box<Self>>) -> MediaSourceStream
    where
        Self: Sync + Send + 'static,
    {
        MediaSourceStream {
            stream: self,
            buffer: Arc::new(RwLock::new(BytesMut::new())),
            position: Arc::new(RwLock::new(0)),
        }
    }
}

pub struct MediaSourceStream {
    stream: Pin<Box<dyn Stream + Sync + Send>>,
    buffer: Arc<RwLock<BytesMut>>,
    position: Arc<RwLock<u64>>,
}

impl MediaSourceStream {
    async fn read_async(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let opt_bytes = if self.buffer.read().await.is_empty() {
            either::Left(
                self.stream
                    .chunk()
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
            )
        } else {
            either::Right(())
        };

        let chunk = match opt_bytes {
            either::Left(Some(chunk)) => Some(chunk),
            either::Left(None) => return Ok(0), // End of stream
            either::Right(_) => None,
        };

        let mut buffer = self.buffer.write().await;
        let mut position = self.position.write().await;

        if let Some(chunk) = chunk {
            buffer.extend_from_slice(&chunk);
        }

        let len = std::cmp::min(buf.len(), buffer.len());
        buf[..len].copy_from_slice(&buffer[..len]);
        buffer.advance(len);
        *position += len as u64;

        Ok(len)
    }
}

impl Read for MediaSourceStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rt = Runtime::new()?; // Create a new Tokio runtime
        let fut = self.read_async(buf); // Call your async function

        // Block on the future to get the result
        rt.block_on(fut)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl Seek for MediaSourceStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::End(offset) => {
                let len = self.byte_len().ok_or(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid seek position",
                ))?;
                let new_position = len as i64 + offset;
                if new_position < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid seek position",
                    ));
                }
                let mut position = self.position.blocking_write();
                *position = new_position as u64;
                Ok(*position)
            }
            SeekFrom::Start(offset) => {
                let mut position = self.position.blocking_write();
                *position = offset;
                Ok(*position)
            }
            SeekFrom::Current(offset) => {
                let mut position = self.position.blocking_write();
                let new_position = (*position as i64) + offset;
                if new_position < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid seek position",
                    ));
                }
                *position = new_position as u64;
                Ok(*position)
            }
        }
    }
}

impl MediaSource for MediaSourceStream {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.stream.content_length() as u64)
    }
}
