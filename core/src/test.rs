/*
 * Created on Fri Sep 17 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use futures::executor::block_on;

use crate::backend::StoryboardBackend;

pub fn create_test_backend() -> StoryboardBackend {
    let backend = block_on(StoryboardBackend::init(Default::default())).unwrap();

    backend
}
