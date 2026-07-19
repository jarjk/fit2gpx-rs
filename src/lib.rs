//! fit2gpx
//!
//! A simple, yet powerful fit to gpx converter,
//! capable of adding elevation while conversion from `HGT` DTM data.
//!
//! Uses `rayon` for multithreaded execution.
//!
//! # Usage
//!
//! ```rust
#![doc = include_str!("../examples/demo.rs")]
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(unsafe_code)]

/// universal Result, but not sendable
pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "elevation")]
pub mod elevation;
pub use fit::Fit;
pub mod fit;
mod utils;
