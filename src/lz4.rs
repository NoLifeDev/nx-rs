// Copyright © 2015-2018, Peter Atashian
use std::os::raw::{c_int};
extern "C" {
    fn LZ4_decompress_safe(
        source: *const u8, dest: *mut u8, compressedSize: c_int, maxDecompressedSize: c_int,
    ) -> c_int;
}
pub fn decompress(source: &[u8], dest: &mut [u8]) -> Result<usize, c_int> {
    let ret = unsafe {
        LZ4_decompress_safe(source.as_ptr(), dest.as_mut_ptr(), source.len() as c_int,
            dest.len() as c_int)
    };
    if ret < 0 { Err(ret) }
    else { Ok(ret as usize) }
}
