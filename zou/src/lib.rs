extern crate ansi_term;
extern crate hyper;
extern crate hyper_openssl;
extern crate pbr;
extern crate rayon;

#[macro_use]
pub mod logs;

pub mod authorization;
pub mod bench;
pub mod cargo_helper;
pub mod client;
pub mod contentlength;
pub mod download;
pub mod filesize;
pub mod http_version;
pub mod protocol;
pub mod response;
pub mod util;
pub mod write;

use std::{
    sync::{
        Arc, 
        Mutex
    }
};

/// Представляет собой число байт как `u64` 
pub type BytesLength = u64;
/// Represents a 'chunk', which is just a piece of bytes.
type Chunk = Vec<u8>;
/// Represents a list of chunks
pub type Chunks = Vec<Chunk>;
/// Represents a shared mutable reference of chunks
pub type SChunks = Arc<Mutex<Chunks>>;
/// Represents an URL
pub type URL<'a> = &'a str;
/// MirrorsList is an alias that contain fast URLs to download the file
type MirrorsList<'a> = Vec<URL<'a>>;
