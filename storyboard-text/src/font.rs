/*
 * Created on Tue Jun 14 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::slice;
use std::{
    borrow::{Borrow, Cow},
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    pin::Pin,
};

use allsorts::tables::FontTableProvider;
use ttf_parser::{Face, FaceParsingError, Tag};

pub struct Font<'a> {
    data: &'a [u8],
    index: u32,
    face: Face<'a>,
    file_hash: u64,
}

impl<'a> Font<'a> {
    pub fn from_slice(data: &'a [u8], index: u32) -> Result<Self, FaceParsingError> {
        let face = Face::from_slice(data, index)?;

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);

        Ok(Self {
            data,
            index,
            face,
            file_hash: hasher.finish(),
        })
    }

    pub const fn file_hash(this: &Self) -> u64 {
        this.file_hash
    }
}

impl<'a> Deref for Font<'a> {
    type Target = Face<'a>;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

impl<'a> ToOwned for Font<'a> {
    type Owned = OwnedFont;

    fn to_owned(&self) -> Self::Owned {
        OwnedFont::from_vec(self.data.into(), self.index).unwrap()
    }
}

impl Debug for Font<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("face", &self.face)
            .field("file_hash", &self.file_hash)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct OwnedFont {
    _data: Pin<Box<[u8]>>,
    font: Font<'static>,
}

impl OwnedFont {
    pub fn from_vec(data: Vec<u8>, index: u32) -> Result<Self, FaceParsingError> {
        let data = Pin::new(data.into_boxed_slice());

        // SAFETY:: The original data does not move in struct and cannot be accessed outside.
        let font = Font::from_slice(
            unsafe { slice::from_raw_parts(data.as_ptr(), data.len()) },
            index,
        )?;

        Ok(Self { _data: data, font })
    }
}

impl<'a> Borrow<Font<'a>> for OwnedFont {
    fn borrow(&self) -> &Font<'a> {
        &self.font
    }
}

impl Deref for OwnedFont {
    type Target = Font<'static>;

    fn deref(&self) -> &Self::Target {
        &self.font
    }
}

impl FontTableProvider for Font<'_> {
    fn table_data<'a>(
        &'a self,
        tag: u32,
    ) -> Result<Option<Cow<'a, [u8]>>, allsorts::error::ParseError> {
        Ok(self
            .face
            .table_data(Tag(tag))
            .map(|data| Cow::Borrowed(data)))
    }

    fn has_table<'a>(&'a self, tag: u32) -> bool {
        self.face.table_data(Tag(tag)).is_some()
    }
}
