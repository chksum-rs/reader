//! This crate provides a convenient interface for calculating hash digests on the fly while reading data from a reader. It supports various hash algorithms, and the library is designed to be easy to use.
//!
//! # Setup
//!
//! To use this crate, add the following entry to your `Cargo.toml` file in the `dependencies` section:
//!
//! ```toml
//! [dependencies]
//! chksum-reader = "0.0.0"
//! ```
//!
//! Alternatively, you can use the [`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html) subcommand:
//!
//! ```sh
//! cargo add chksum-reader
//! ```     
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

#![forbid(unsafe_code)]

use std::io::{self, BufRead, Read};

use chksum_core::Hash;

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
