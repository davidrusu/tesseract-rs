extern crate leptonica_sys;
extern crate tesseract_sys;

use leptonica_sys::{pixFreeData, pixRead, pixReadMem};
use std::convert::AsRef;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use tesseract_sys::{
    TessBaseAPI, TessBaseAPICreate, TessBaseAPIDelete, TessBaseAPIGetUTF8Text, TessBaseAPIInit3,
    TessBaseAPIRecognize, TessBaseAPISetImage, TessBaseAPISetImage2,
    TessBaseAPISetSourceResolution, TessBaseAPISetVariable, TessDeleteText,
};

pub struct Tesseract {
    raw: *mut TessBaseAPI,
}

impl Drop for Tesseract {
    fn drop(&mut self) {
        unsafe { TessBaseAPIDelete(self.raw) }
    }
}

pub struct TesseractText {
    raw: *const c_char,
}

impl Drop for TesseractText {
    fn drop(&mut self) {
        unsafe { TessDeleteText(self.raw) }
    }
}

impl TesseractText {
    unsafe fn new(raw: *const c_char) -> Self {
        Self { raw }
    }
}

impl AsRef<CStr> for TesseractText {
    fn as_ref(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw) }
    }
}

impl Default for Tesseract {
    fn default() -> Self {
        Self::new()
    }
}

impl Tesseract {
    pub fn new() -> Tesseract {
        Tesseract {
            raw: unsafe { TessBaseAPICreate() },
        }
    }
    pub fn set_lang(&mut self, language: &CStr) -> i32 {
        unsafe { TessBaseAPIInit3(self.raw, ptr::null(), language.as_ptr()) }
    }
    pub fn set_image(&mut self, filename: &CStr) {
        unsafe {
            let img = pixRead(filename.as_ptr());
            TessBaseAPISetImage2(self.raw, img);
            pixFreeData(img);
        }
    }
    pub fn set_frame(
        &mut self,
        frame_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) {
        unsafe {
            TessBaseAPISetImage(
                self.raw,
                frame_data.as_ptr(),
                width,
                height,
                bytes_per_pixel,
                bytes_per_line,
            );
        }
    }
    pub fn set_image_from_mem(&mut self, img: &[u8]) {
        unsafe {
            let img = pixReadMem(img.as_ptr(), img.len());
            TessBaseAPISetImage2(self.raw, img);
            pixFreeData(img);
        }
    }

    pub fn set_source_resolution(&mut self, ppi: i32) {
        unsafe {
            TessBaseAPISetSourceResolution(self.raw, ppi);
        }
    }

    pub fn set_variable(&mut self, name: &CStr, value: &CStr) -> i32 {
        unsafe { TessBaseAPISetVariable(self.raw, name.as_ptr(), value.as_ptr()) }
    }
    pub fn recognize(&mut self) -> i32 {
        unsafe { TessBaseAPIRecognize(self.raw, ptr::null_mut()) }
    }
    pub fn get_text(&self) -> TesseractText {
        unsafe {
            let cs_value = TessBaseAPIGetUTF8Text(self.raw);
            TesseractText::new(cs_value)
        }
    }
}

pub fn ocr(filename: &CStr, language: &CStr) -> TesseractText {
    let mut cube = Tesseract::new();
    cube.set_lang(language);
    cube.set_image(filename);
    cube.recognize();
    cube.get_text()
}

pub fn ocr_from_frame(
    frame_data: &[u8],
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    bytes_per_line: i32,
    language: &CStr,
) -> TesseractText {
    let mut cube = Tesseract::new();
    cube.set_lang(language);
    cube.set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line);
    cube.recognize();
    cube.get_text()
}

#[test]
fn ocr_test() {
    use std::ffi::CString;

    assert_eq!(
        ocr(
            &CString::new("../img.png").unwrap(),
            &CString::new("eng").unwrap()
        )
        .as_ref()
        .to_str(),
        Ok(include_str!("../../img.txt"))
    );
}

#[test]
fn ocr_from_frame_test() {
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Read;

    let mut img = File::open("../img.tiff").unwrap();
    let mut buffer = Vec::new();
    img.read_to_end(&mut buffer).unwrap();

    assert_eq!(
        ocr_from_frame(
            &buffer,
            2256,
            324,
            3,
            2256 * 3,
            &CString::new("eng").unwrap()
        )
        .as_ref()
        .to_str(),
        Ok(include_str!("../../img.txt"))
    );
}

#[test]
fn ocr_from_mem_with_ppi() {
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Read;

    let mut img = File::open("../img.tiff").unwrap();
    let mut buffer = Vec::new();
    img.read_to_end(&mut buffer).unwrap();

    let mut cube = Tesseract::new();
    cube.set_lang(&CString::new("eng").unwrap());
    cube.set_image_from_mem(&buffer);

    cube.set_source_resolution(70);
    assert_eq!(
        cube.get_text().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    );
}

#[test]
fn expanded_test() {
    use std::ffi::CString;

    let mut cube = Tesseract::new();
    cube.set_lang(&CString::new("eng").unwrap());
    cube.set_image(&CString::new("../img.png").unwrap());
    cube.recognize();
    assert_eq!(
        cube.get_text().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    )
}
