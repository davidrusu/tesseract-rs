extern crate leptonica_sys;
extern crate tesseract_sys;

use leptonica_sys::{pixFreeData, pixRead, pixReadMem};
use std::convert::AsRef;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use tesseract_sys::{
    TessBaseAPICreate, TessBaseAPIDelete, TessBaseAPIGetUTF8Text, TessBaseAPIInit3,
    TessBaseAPIRecognize, TessBaseAPISetImage, TessBaseAPISetImage2,
    TessBaseAPISetSourceResolution, TessBaseAPISetVariable, TessDeleteText,
};

pub struct Pix {
    raw: *mut leptonica_sys::Pix,
}

impl Drop for Pix {
    fn drop(&mut self) {
        unsafe {
            pixFreeData(self.raw);
        }
    }
}

impl Pix {
    unsafe fn new(raw: *mut leptonica_sys::Pix) -> Self {
        Self { raw }
    }

    /// Wrapper for [`pixRead`](https://tpgit.github.io/Leptonica/leptprotos_8h.html#a84634846cbb5e01df667d6e9241dfc53)
    ///
    /// Read an image from a filename
    fn read(filename: &CStr) -> Result<Self, ()> {
        let ptr = unsafe { pixRead(filename.as_ptr()) };
        if ptr.is_null() {
            Err(())
        } else {
            Ok(unsafe { Self::new(ptr) })
        }
    }

    /// Wrapper for [`pixReadMem`](https://tpgit.github.io/Leptonica/leptprotos_8h.html#a027a927dc3438192e3bdae8c219d7f6a)
    ///
    /// Read an image from memory
    pub fn read_mem(img: &[u8]) -> Result<Self, ()> {
        let ptr = unsafe { pixReadMem(img.as_ptr(), img.len()) };
        if ptr.is_null() {
            Err(())
        } else {
            Ok(unsafe { Self::new(ptr) })
        }
    }
}

pub struct TessBaseAPI {
    raw: *mut tesseract_sys::TessBaseAPI,
}

impl Drop for TessBaseAPI {
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

impl Default for TessBaseAPI {
    fn default() -> Self {
        Self::new()
    }
}

impl TessBaseAPI {
    pub fn new() -> TessBaseAPI {
        TessBaseAPI {
            raw: unsafe { TessBaseAPICreate() },
        }
    }

    /// Wrapper for [`Init`](https://tesseract-ocr.github.io/tessapi/5.x/a02438.html#a96899e8e5358d96752ab1cfc3bc09f3e)
    ///
    /// Start tesseract
    ///
    /// TODO: implement the additional parameters.
    pub fn initialize(&self, datapath: Option<&CStr>, language: Option<&CStr>) -> Result<(), ()> {
        let ret = unsafe {
            TessBaseAPIInit3(
                self.raw,
                datapath.map(CStr::as_ptr).unwrap_or_else(ptr::null),
                language.map(CStr::as_ptr).unwrap_or_else(ptr::null),
            )
        };
        if ret == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Wrapper for [`SetImage-2`](https://tesseract-ocr.github.io/tessapi/5.x/a02438.html#a0c4c7f05fd58b3665b123232a05545ad)
    pub fn set_image_2(&mut self, pix: &Pix) {
        unsafe {
            TessBaseAPISetImage2(self.raw, pix.raw);
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

    pub fn set_source_resolution(&mut self, ppi: i32) {
        unsafe {
            TessBaseAPISetSourceResolution(self.raw, ppi);
        }
    }

    pub fn set_variable(&mut self, name: &CStr, value: &CStr) -> i32 {
        unsafe { TessBaseAPISetVariable(self.raw, name.as_ptr(), value.as_ptr()) }
    }
    /// Wrapper for [`Recognize`](https://tesseract-ocr.github.io/tessapi/5.x/a02438.html#a0e4065c20b142d69a2324ee0c74ae0b0)
    ///
    /// Recognize the image. Returns `Ok(())` on success and `Err(())` otherwise.
    /// It is currently unclear to me what would make it error.
    ///
    /// It could take a progress argument (`monitor`). If there is appetite for this, let me know and I could try and implement it.
    pub fn recognize(&mut self) -> Result<(), ()> {
        let ret = unsafe { TessBaseAPIRecognize(self.raw, ptr::null_mut()) };
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
        let ptr = unsafe { TessBaseAPIGetUTF8Text(self.raw) };
        if ptr.is_null() {
            Err(())
        } else {
            Ok(unsafe { TesseractText::new(ptr) })
        }
    }
}

pub fn ocr(filename: &CStr, language: &CStr) -> TesseractText {
    let mut cube = TessBaseAPI::new();
    cube.initialize(None, Some(language)).unwrap();
    let image = Pix::read(filename).unwrap();
    cube.set_image_2(&image);
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
    let mut cube = TessBaseAPI::new();
    cube.initialize(None, Some(language)).unwrap();
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

    assert_eq!(
        ocr_from_frame(
            include_bytes!("../../img.tiff"),
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

    let pix = Pix::read_mem(include_bytes!("../../img.tiff")).unwrap();

    let mut cube = TessBaseAPI::new();
    cube.initialize(None, Some(&CString::new("eng").unwrap()))
        .unwrap();
    cube.set_image_2(&pix);

    cube.set_source_resolution(70);
    assert_eq!(
        cube.get_text().unwrap().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    );
}

#[test]
fn expanded_test() {
    use std::ffi::CString;

    let mut cube = TessBaseAPI::new();
    cube.initialize(None, Some(&CString::new("eng").unwrap()))
        .unwrap();
    let pix = Pix::read(&CString::new("../img.png").unwrap()).unwrap();
    cube.set_image_2(&pix);
    cube.recognize().unwrap();
    assert_eq!(
        cube.get_text().unwrap().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    )
}

#[test]
fn setting_image_without_initializing_test() {
    let mut cube = TessBaseAPI::new();
    let pix = Pix::read_mem(include_bytes!("../../img.tiff")).unwrap();
    cube.set_image_2(&pix);
    assert_eq!(cube.recognize(), Err(()));
    assert!(cube.get_text().is_err());
}
