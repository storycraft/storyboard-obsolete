use core::slice;
use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    pin::Pin,
    sync::Arc,
};

use ttf_parser::{Face, FaceParsingError};

#[derive(Clone)]
pub struct Font {
    font_hash: u64,
    face: Arc<(Pin<Cow<'static, [u8]>>, Face<'static>)>,
}

impl Font {
    pub fn new(data: Cow<'static, [u8]>, index: u32) -> Result<Self, FaceParsingError> {
        let data = Pin::new(data);

        // data is pinned and the reference of data never move
        let face = Face::from_slice(
            unsafe { slice::from_raw_parts(data.as_ptr(), data.len()) },
            index,
        )?;

        let mut hasher = DefaultHasher::new();
        index.hash(&mut hasher);
        data.hash(&mut hasher);

        Ok(Self {
            face: Arc::new((data, face)),
            font_hash: hasher.finish(),
        })
    }

    pub const fn font_hash(this: &Self) -> u64 {
        this.font_hash
    }
}

impl Deref for Font {
    type Target = Face<'static>;

    fn deref(&self) -> &Self::Target {
        &self.face.1
    }
}

impl Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("face", &self.face.1)
            .field("file_hash", &self.font_hash)
            .finish_non_exhaustive()
    }
}
