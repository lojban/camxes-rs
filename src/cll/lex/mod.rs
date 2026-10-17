//! Morphological tokens — output of the hybrid pipeline.
//!
//! These feed CLL Ch. 21 terminals (`BRIVLA`, `CMENE`, and cmavo selma'o),
//! which are **post-lex**. Selma'o assignment for cmavo is out of morphology
//! scope (see CLL Ch. 20–21).

pub mod stream;
pub mod token;

pub use stream::{
    morph_parse_stream_piece, segment_continuous, segment_orthographic, segment_stream,
    to_lex_token, OverflowPolicy, StreamMode, StreamPush, StreamResult, StreamToken,
    StreamingSegmenter, DEFAULT_MAX_LOOKAHEAD,
};
pub use token::*;
