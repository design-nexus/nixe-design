//! Shared media-card construction from Media Player. Apps supply image loading and metadata.
use crate::widgets;
use gtk::{gdk, prelude::*};

#[derive(Clone)]
pub struct Artwork {
    pub root: gtk::Overlay,
    pub picture: gtk::Picture,
    pub placeholder: gtk::Image,
}
impl Artwork {
    pub fn new(width: i32, height: i32, icon: &str) -> Self {
        let root = gtk::Overlay::new();
        root.add_css_class("art");
        root.set_overflow(gtk::Overflow::Hidden);
        root.set_size_request(width, height);
        root.set_halign(gtk::Align::Start);
        root.set_valign(gtk::Align::Start);
        let placeholder = gtk::Image::from_icon_name(icon);
        placeholder.add_css_class("art-placeholder");
        placeholder.set_pixel_size((width.min(height) / 3).clamp(16, 72));
        placeholder.set_size_request(width, height);
        root.set_child(Some(&placeholder));
        let picture = gtk::Picture::new();
        picture.set_content_fit(gtk::ContentFit::Cover);
        picture.set_can_shrink(true);
        picture.set_size_request(width, height);
        picture.set_visible(false);
        root.add_overlay(&picture);
        Self { root, picture, placeholder }
    }
    pub fn set_texture(&self, texture: Option<&gdk::Texture>) {
        self.picture.set_paintable(texture);
        self.picture.set_visible(texture.is_some());
        self.placeholder.set_visible(texture.is_none());
    }
}
pub fn progress_strip() -> gtk::ProgressBar {
    let p = gtk::ProgressBar::new();
    p.add_css_class("watch-progress");
    p.set_valign(gtk::Align::End);
    p.set_can_target(false);
    p.set_visible(false);
    p
}

pub fn set_progress(strip: &gtk::ProgressBar, fraction: f64) {
    strip.set_fraction(fraction);
    strip.set_visible(fraction > 0.0);
}

/// A play glyph in the middle of a picture, shown while the pointer is over
/// its card.
pub fn hover_play() -> gtk::Image {
    let play = gtk::Image::from_icon_name("media-playback-start-symbolic");
    play.add_css_class("hover-play");
    play.set_pixel_size(24);
    play.set_halign(gtk::Align::Center);
    play.set_valign(gtk::Align::Center);
    play.set_can_target(false);
    play
}

/// A small round mark in a picture's corner: a tick when watched, or a count.
pub fn badge() -> gtk::Label {
    let l = gtk::Label::new(None);
    l.add_css_class("badge");
    l.set_halign(gtk::Align::End);
    l.set_valign(gtk::Align::Start);
    l.set_can_target(false);
    l.set_visible(false);
    l
}

pub fn set_badge(badge: &gtk::Label, watched: bool, count: usize) {
    if watched {
        badge.set_text("✓");
        badge.add_css_class("watched");
        badge.set_visible(true);
    } else {
        badge.remove_css_class("watched");
        badge.set_text(&count.to_string());
        badge.set_visible(count > 0);
    }
}

#[derive(Clone)]
pub struct PosterCard {
    pub root: gtk::Box,
    pub strip: gtk::ProgressBar,
    pub badge: gtk::Label,
    pub title: gtk::Label,
    pub meta: gtk::Label,
}
impl PosterCard {
    pub fn new(art: &gtk::Overlay, width: i32) -> Self {
        let root = widgets::vbox(6);
        root.add_css_class("poster-card");
        let strip = progress_strip();
        art.add_overlay(&strip);
        let badge = badge();
        art.add_overlay(&badge);
        art.add_overlay(&hover_play());
        root.append(art);
        let title = widgets::label("", "card-title");
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title.set_max_width_chars(1);
        title.set_size_request(width, -1);
        let meta = widgets::label("", "card-meta");
        meta.set_ellipsize(gtk::pango::EllipsizeMode::End);
        meta.set_max_width_chars(1);
        meta.set_size_request(width, -1);
        root.append(&title);
        root.append(&meta);
        Self { root, strip, badge, title, meta }
    }
}
pub const CSS: &str = include_str!("media.css");
