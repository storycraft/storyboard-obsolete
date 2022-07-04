use std::error::Error;

use rustybuzz::{Face, UnicodeBuffer};
use storyboard_text::layout::SpanLayout;

pub static FONT: &[u8] = include_bytes!("./NotoSansCjkKr-Regular.otf");

#[test]
fn layout_test() -> Result<(), Box<dyn Error>> {
    let face = Face::from_slice(FONT, 0).unwrap();

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str("hello world");

    let layout = SpanLayout::shape_from_buffer(&face, 16.0, buffer);

    for info in layout.iter() {
        println!("{info:?}");
    }

    Ok(())
}
