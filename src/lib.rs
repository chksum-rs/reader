//! This crate provides a convenient interface for calculating hash digests on the fly while reading data from a reader. It supports various hash algorithms, and the library is designed to be easy to use.
//!
//! # Setup
//!
//! To use this crate, add the following entry to your `Cargo.toml` file in the `dependencies` section:
//!
//! ```toml
//! [dependencies]
//! chksum-reader = "0.1.0"
//! ```
//!
//! Alternatively, you can use the [`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html) subcommand:
//!
//! ```sh
//! cargo add chksum-reader
//! ```     
//!
//! # Features
//!
//! ## Asynchronous Runtime
//!
//! * `async-runtime-tokio`: Enables async interface for Tokio runtime.
//!
//! By default, neither of these features is enabled.
//!
//! # Usage
//!
//! ```rust,ignore
//! use std::io::{self, Read};
//!
//! use chksum_md5::MD5;
//! use chksum_reader::Reader;
//!
//! fn main() -> io::Result<()> {
//!     // Create a new reader with the MD5 hash algorithm
//!     let mut reader = Reader::<_, MD5>::new(io::stdin());
//!
//!     // Read data from the reader
//!     let mut buffer = Vec::new();
//!     reader.read_to_end(&mut buffer)?;
//!
//!     // Get the calculated digest
//!     let digest = reader.digest();
//!
//!     // Print the digest (hex representation)
//!     println!("Digest: {}", digest.to_hex_lowercase());
//!
//!     Ok(())
//! }
//! ```
//!
//! # Implementations
//!
//! This crate should be used along with a hash implementation crate.
//!  
//! Various crates implement their own [`Reader`], which can be enabled with the `reader` Cargo feature.
//!
//! # License
//!
//! This crate is licensed under the MIT License.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]

use std::io::{self, BufRead, Read};
#[cfg(feature = "async-runtime-tokio")]
use std::pin::{pin, Pin};
#[cfg(feature = "async-runtime-tokio")]
use std::task::{Context, Poll};

use chksum_core::Hash;
#[cfg(feature = "async-runtime-tokio")]
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

/// Creates new [`Reader`].
pub fn new<R, H>(inner: R) -> Reader<R, H>
where
    R: Read,
    H: Hash,
{
    Reader::new(inner)
}

/// Creates new [`Reader`] with provided hash.
pub fn with_hash<R, H>(inner: R, hash: H) -> Reader<R, H>
where
    R: Read,
    H: Hash,
{
    Reader::with_hash(inner, hash)
}

#[cfg(feature = "async-runtime-tokio")]
/// Creates new [`AsyncReader`].
pub fn async_new<R, H>(inner: R) -> AsyncReader<R, H>
where
    R: AsyncReadExt,
    H: Hash,
{
    AsyncReader::new(inner)
}

#[cfg(feature = "async-runtime-tokio")]
/// Creates new [`AsyncReader`] with provided hash.
pub fn async_with_hash<R, H>(inner: R, hash: H) -> AsyncReader<R, H>
where
    R: AsyncReadExt,
    H: Hash,
{
    AsyncReader::with_hash(inner, hash)
}

/// Wraps a reader and calculates the hash digest on the fly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reader<R, H>
where
    R: Read,
    H: Hash,
{
    inner: R,
    hash: H,
}

impl<R, H> Reader<R, H>
where
    R: Read,
    H: Hash,
{
    /// Creates new [`Reader`].
    pub fn new(inner: R) -> Self {
        let hash = H::default();
        Self::with_hash(inner, hash)
    }

    /// Creates new [`Reader`] with provided hash.
    #[must_use]
    pub const fn with_hash(inner: R, hash: H) -> Self {
        Self { inner, hash }
    }

    /// Unwraps this [`Reader`], returning the underlying reader.
    #[must_use]
    pub fn into_inner(self) -> R {
        let Self { inner, .. } = self;
        inner
    }

    /// Returns calculated hash digest.
    #[must_use]
    pub fn digest(&self) -> H::Digest {
        self.hash.digest()
    }
}

impl<R, H> Read for Reader<R, H>
where
    R: Read,
    H: Hash,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.hash.update(&buf[..n]);
        Ok(n)
    }
}

impl<R, H> BufRead for Reader<R, H>
where
    R: BufRead,
    H: Hash,
{
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }

    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }
}

/// Wraps a reader and calculates the hash digest on the fly.
#[cfg(feature = "async-runtime-tokio")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsyncReader<R, H>
where
    R: AsyncReadExt,
    H: Hash,
{
    inner: R,
    hash: H,
}

#[cfg(feature = "async-runtime-tokio")]
impl<R, H> AsyncReader<R, H>
where
    R: AsyncReadExt,
    H: Hash,
{
    /// Creates new [`AsyncReader`].
    pub fn new(inner: R) -> Self {
        let hash = H::default();
        Self::with_hash(inner, hash)
    }

    /// Creates new [`AsyncReader`] with provided hash.
    #[must_use]
    pub const fn with_hash(inner: R, hash: H) -> Self {
        Self { inner, hash }
    }

    /// Unwraps this [`AsyncReader`], returning the underlying reader.
    #[must_use]
    pub fn into_inner(self) -> R {
        let Self { inner, .. } = self;
        inner
    }

    /// Returns calculated hash digest.
    #[must_use]
    pub fn digest(&self) -> H::Digest {
        self.hash.digest()
    }
}

#[cfg(feature = "async-runtime-tokio")]
impl<R, H> AsyncRead for AsyncReader<R, H>
where
    R: AsyncRead + Unpin,
    H: Hash + Unpin,
{
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let Self { inner, hash } = self.get_mut();
        match pin!(inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                hash.update(buf.filled());
                Poll::Ready(Ok(()))
            },
            poll => poll,
        }
    }
}
