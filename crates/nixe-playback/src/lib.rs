//! Shared libmpv client and Wayland-compatible GTK video rendering.
//!
//! Create and use the client on the GTK main thread. The renderer currently
//! supports one player per process. Call `video::shutdown` before dropping its
//! `mpv::Mpv`; application-specific queues and watch policy belong to consumers.
pub mod controls;
pub mod mpv;
pub mod timeline;
pub mod video;
