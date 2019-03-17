use std::fmt::{self, Display};

use crate::utils::notify::NotificationLevel;

#[derive(Debug)]
pub enum Notification<'a> {
    /// Received the Content-Length of the to-be downloaded data.
    DownloadContentLengthReceived(u64),
    /// Received some data.
    DownloadDataReceived(&'a [u8]),
    /// Download has finished.
    DownloadFinished,
    ResumingPartialDownload,
}

impl<'a> Notification<'a> {
    pub fn level(&self) -> NotificationLevel {
        use self::Notification::*;
        match *self {
            DownloadContentLengthReceived(_)
            | DownloadDataReceived(_)
            | DownloadFinished
            | ResumingPartialDownload => NotificationLevel::Verbose,
        }
    }
}

impl<'a> Display for Notification<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        use self::Notification::*;
        match *self {
            DownloadContentLengthReceived(len) => write!(f, "download size is: '{}'", len),
            DownloadDataReceived(data) => write!(f, "received some data of size {}", data.len()),
            DownloadFinished => write!(f, "download finished"),
            ResumingPartialDownload => write!(f, "resuming partial download"),
        }
    }
}