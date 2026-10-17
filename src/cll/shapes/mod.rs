//! CV shape helpers and shape-layer recognition — CLL §4.1+.
//!
//! [`recognize()`] is the public CFG overgenerator entry point.

pub mod cv;
pub mod recognize;

pub use cv::*;
pub use recognize::*;
