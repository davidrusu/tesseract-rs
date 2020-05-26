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

pub struct TesseractInitialized(Tesseract);

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

    /// Wrapper for [`Init`](https://tesseract-ocr.github.io/tessapi/5.x/a02438.html#a96899e8e5358d96752ab1cfc3bc09f3e)
    ///
    /// Start tesseract
    ///
    /// TODO: implement the additional parameters.
    pub fn initialize(
        self,
        datapath: Option<&CStr>,
        language: Option<&CStr>,
    ) -> Result<TesseractInitialized, ()> {
        let ret = unsafe {
            TessBaseAPIInit3(
                self.raw,
                datapath.map(CStr::as_ptr).unwrap_or_else(ptr::null),
                language.map(CStr::as_ptr).unwrap_or_else(ptr::null),
            )
        };
        if ret == 0 {
            Ok(TesseractInitialized(self))
        } else {
            Err(())
        }
    }
}
impl TesseractInitialized {
    pub fn set_image(&mut self, filename: &CStr) {
        unsafe {
            let img = pixRead(filename.as_ptr());
            TessBaseAPISetImage2(self.0.raw, img);
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
                self.0.raw,
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
            TessBaseAPISetImage2(self.0.raw, img);
            pixFreeData(img);
        }
    }

    pub fn set_source_resolution(&mut self, ppi: i32) {
        unsafe {
            TessBaseAPISetSourceResolution(self.0.raw, ppi);
        }
    }

    pub fn set_variable(&mut self, name: &CStr, value: &CStr) -> i32 {
        unsafe { TessBaseAPISetVariable(self.0.raw, name.as_ptr(), value.as_ptr()) }
    }
    /// Wrapper for [`Recognize`](https://tesseract-ocr.github.io/tessapi/5.x/a02438.html#a0e4065c20b142d69a2324ee0c74ae0b0)
    ///
    /// Recognize the image. Returns `Ok(())` on success and `Err(())` otherwise.
    /// It is currently unclear to me what would make it error.
    ///
    /// It could take a progress argument (`monitor`). If there is appetite for this, let me know and I could try and implement it.
    pub fn recognize(&mut self) -> Result<(), ()> {
        let ret = unsafe { TessBaseAPIRecognize(self.0.raw, ptr::null_mut()) };
        match ret {
            0 => Ok(()),
            _ => Err(()),
        }
    }
    /// Wrapper for [`GetUTF8Text`](https://tesseract-ocr.github.io/tessapi/5.x/a02438.html#a115ef656f83352ba608b4f0bf9cfa2c4)
    ///
    /// Get the text out of an image.
    ///
    /// Can return an error (null pointer), but it is not clear to me what would cause this.
    ///
    /// This will implicitly call `recognize` if required.
    pub fn get_text(&self) -> Result<TesseractText, ()> {
        let cs_value = unsafe { TessBaseAPIGetUTF8Text(self.0.raw) };
        if cs_value.is_null() {
            Err(())
        } else {
            Ok(unsafe { TesseractText::new(cs_value) })
        }
    }
}

pub fn ocr(filename: &CStr, language: &CStr) -> TesseractText {
    let mut cube = Tesseract::new().initialize(None, Some(language)).unwrap();
    cube.set_image(filename);
    cube.recognize().unwrap();
    cube.get_text().unwrap()
}

pub fn ocr_from_frame(
    frame_data: &[u8],
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    bytes_per_line: i32,
    language: &CStr,
) -> TesseractText {
    let mut cube = Tesseract::new().initialize(None, Some(language)).unwrap();
    cube.set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line);
    cube.recognize().unwrap();
    cube.get_text().unwrap()
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

    let mut cube = Tesseract::new()
        .initialize(None, Some(&CString::new("eng").unwrap()))
        .unwrap();
    cube.set_image_from_mem(&buffer);

    cube.set_source_resolution(70);
    assert_eq!(
        cube.get_text().unwrap().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    );
}

#[test]
fn expanded_test() {
    use std::ffi::CString;

    let mut cube = Tesseract::new()
        .initialize(None, Some(&CString::new("eng").unwrap()))
        .unwrap();
    cube.set_image(&CString::new("../img.png").unwrap());
    cube.recognize().unwrap();
    assert_eq!(
        cube.get_text().unwrap().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    )
}
