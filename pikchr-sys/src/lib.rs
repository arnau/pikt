//! The raw bindings for `pikchr.c`.
//!
//! See [`render`] for a safe wrapper.
//!
//! Using [`pikchr`] will require manually freeing the buffer.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use libc::free;
    use std::ffi::{CStr, CString};
    use std::os::raw::*;

    #[test]
    fn simple_raw_box() {
        let input = "box \"pikchr\"";
        let expected = "<svg xmlns='http://www.w3.org/2000/svg' viewBox=\"0 0 112.32 76.32\">\n<path d=\"M2,74L110,74L110,2L2,2Z\"  style=\"fill:none;stroke-width:2.16;stroke:rgb(0,0,0);\" />\n<text x=\"56\" y=\"38\" text-anchor=\"middle\" fill=\"rgb(0,0,0)\" dominant-baseline=\"central\">pikchr</text>\n</svg>\n";

        let source = CString::new(input).unwrap();
        let flags = PIKCHR_PLAINTEXT_ERRORS;
        let mut width: c_int = 0;
        let mut height: c_int = 0;

        let res: *mut c_char = unsafe {
            pikchr(
                source.as_ptr() as *const c_char,
                std::ptr::null(),
                flags,
                &mut width as *mut c_int,
                &mut height as *mut c_int,
            )
        };
        let cstr = unsafe { CStr::from_ptr(res) };
        let actual = String::from_utf8_lossy(cstr.to_bytes()).into_owned();

        unsafe { free(res as *mut c_void) };

        assert_eq!(&actual, expected);
    }
}
