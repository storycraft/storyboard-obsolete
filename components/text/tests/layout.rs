use std::error::Error;

use rustybuzz::{Face, UnicodeBuffer};
use storyboard_text::layout::LineLayout;

pub static FONT: &'static [u8] = include_bytes!("./NotoSansCjkKr-Regular.otf");

#[test]
fn layout_test() -> Result<(), Box<dyn Error>> {
    let face = Face::from_slice(FONT, 0).unwrap();

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str("hello world");

    let layout = LineLayout::new_layout(&face, buffer);

    for info in layout.iter(16.0) {
        println!("{info:?}");
    }

    Ok(())
}
