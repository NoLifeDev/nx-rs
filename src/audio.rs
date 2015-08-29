// Copyright © 2015, Peter Atashian
//! Stuff for working with NX nodes

/// Some audio, possibly a sound effect or music
#[derive(Clone, Copy)]
pub struct Audio<'a> {
    data: &'a [u8],
}
impl<'a> Audio<'a> {
    /// Creates an Audio from the supplied data
    pub unsafe fn construct(data: &'a [u8]) -> Audio<'a> {
        Audio { data: data }
    }
    /// Returns the audio data, not including the wz audio header
    pub fn data(&self) -> &[u8] {
        &self.data[82..]
    }
}
