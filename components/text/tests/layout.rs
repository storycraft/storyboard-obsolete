use std::borrow::Cow;

use storyboard_render::wgpu::{Backends, Features, Instance};

use storyboard_render::backend::{BackendOptions, StoryboardBackend};
use storyboard_text::cache::GlyphCache;
use storyboard_text::font::Font;

pub static FONT: &'static [u8] = include_bytes!("./NotoSansCJKkr-Regular.otf");

#[test]
fn test_cache() {
    let backend = pollster::block_on(StoryboardBackend::init(
        &Instance::new(Backends::all()),
        None,
        Features::empty(),
        &BackendOptions::default(),
    ))
    .unwrap();

    println!("backend: {:?}", backend);

    let font = Font::new(Cow::Borrowed(FONT), 0).unwrap();

    let mut cache = GlyphCache::new();

    println!(
        "Batch: {:?}",
        cache.batch_glyphs(
            backend.device(),
            backend.queue(),
            &font,
            "test string"
                .chars()
                .map(|ch| font.glyph_index(ch).unwrap_or_default().0),
            40
        )
    );
}
