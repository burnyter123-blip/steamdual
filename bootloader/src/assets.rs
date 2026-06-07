//! Embedded, build-time-rasterized logo blobs (premultiplied BGRA).
//!
//! `build.rs` emits `logos.rs` into `OUT_DIR` defining `Logo` plus the
//! `STEAMOS` and `WINDOWS` statics. See that file for the pipeline.

include!(concat!(env!("OUT_DIR"), "/logos.rs"));
