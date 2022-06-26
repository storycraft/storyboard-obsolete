use std::error::Error;

use rustybuzz::{Face, UnicodeBuffer};
use storyboard_render::{backend::{BackendOptions, StoryboardBackend}, wgpu::{Backends, Instance}};
use storyboard_text::{cache::GlyphCache};

pub static FONT: &'static [u8] = include_bytes!("./NotoSansCJKkr-Regular.otf");

#[test]
fn layout_test() -> Result<(), Box<dyn Error>> {
    let backend = pollster::block_on(StoryboardBackend::init(
        &Instance::new(Backends::all()),
        None,
        storyboard_render::wgpu::Features::empty(),
        &BackendOptions::default(),
    ))
    .unwrap();
    
    let face = Face::from_slice(FONT, 0).unwrap();
    
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str("Hello world");

    let buffer = rustybuzz::shape(&face, &[], buffer);
    
    let mut cache = GlyphCache::new();

    let mut indices_iter = buffer.glyph_infos().iter().map(|info| info.glyph_id as u16).peekable();
    while let Some(batch) = cache.batch(backend.device(), backend.queue(), &face, &mut indices_iter, 16) {
        println!("batch: {:?}\n", batch);
    }

    Ok(())
}
