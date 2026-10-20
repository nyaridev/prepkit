//! PrepKit's tools, shared by the desktop app and the CLI.
//!
//! - [`pixel_fix`]: fills fully transparent pixels with the nearest edge color, so sprites don't get dark or bright
//!   fringes when they're filtered or scaled.
//! - [`audio`]: normalizes audio (peak or RMS) and optionally converts it, through ffmpeg.
//! - [`ffmpeg`]: finds ffmpeg, or downloads it and adds it to the user's PATH.

pub mod audio;
pub mod batch;
pub mod ffmpeg;
pub mod pixel_fix;
