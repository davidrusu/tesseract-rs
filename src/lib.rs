extern crate thiserror;

use self::thiserror::Error;
use std::ffi::CString;
use std::ffi::NulError;
use std::str;

pub mod plumbing;

#[derive(Debug, Error)]
pub enum SetVariableError {
    #[error("Conversion to CString failed")]
    CStringError(#[from] NulError),
    #[error("TessBaseApi failed to set variable")]
    TessBaseAPISetVariableError(#[from] plumbing::TessBaseAPISetVariableError),
}

pub struct Tesseract(plumbing::TessBaseAPI);

impl Default for Tesseract {
    fn default() -> Self {
        Self::new()
    }
}

impl Tesseract {
    pub fn new() -> Tesseract {
        Tesseract(plumbing::TessBaseAPI::new())
    }
    pub fn set_lang(&mut self, language: &str) -> Result<(), plumbing::TessBaseAPIInitError> {
        self.0.init_2(None, Some(&CString::new(language).unwrap()))
    }
    pub fn set_image(&mut self, filename: &str) {
        let pix = plumbing::Pix::read(&CString::new(filename).unwrap()).unwrap();
        self.0.set_image_2(&pix)
    }
    pub fn set_frame(
        &mut self,
        frame_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) {
        self.0
            .set_image(frame_data, width, height, bytes_per_pixel, bytes_per_line);
    }
    pub fn set_image_from_mem(&mut self, img: &[u8]) -> Result<(), plumbing::PixReadMemError> {
        let pix = plumbing::Pix::read_mem(img)?;
        self.0.set_image_2(&pix);
        Ok(())
    }

    pub fn set_source_resolution(&mut self, ppi: i32) {
        self.0.set_source_resolution(ppi)
    }

    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), SetVariableError> {
        Ok(self
            .0
            .set_variable(&CString::new(name)?, &CString::new(value)?)?)
    }
    pub fn recognize(&mut self) -> Result<(), plumbing::TessBaseAPIRecogniseError> {
        self.0.recognize()
    }
    pub fn get_text(&mut self) -> Result<String, plumbing::TessBaseAPIGetUTF8TextError> {
        Ok(self
            .0
            .get_utf8_text()?
            .as_ref()
            .to_string_lossy()
            .into_owned())
    }
}

pub fn ocr(filename: &str, language: &str) -> String {
    let mut cube = Tesseract::new();
    cube.set_lang(language).unwrap();
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
    language: &str,
) -> String {
    let mut cube = Tesseract::new();
    cube.set_lang(language).unwrap();
    cube.set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line);
    cube.recognize().unwrap();
    cube.get_text().unwrap()
}

#[test]
fn ocr_test() {
    assert_eq!(
        ocr("img.png", "eng"),
        include_str!("../img.txt").to_string()
    );
}

#[test]
fn ocr_from_frame_test() {
    assert_eq!(
        ocr_from_frame(include_bytes!("../img.tiff"), 2256, 324, 3, 2256 * 3, "eng"),
        include_str!("../img.txt").to_string()
    );
}

#[test]
fn ocr_from_mem_with_ppi() {
    let mut cube = Tesseract::new();
    cube.set_lang("eng").unwrap();
    cube.set_image_from_mem(include_bytes!("../img.tiff"))
        .unwrap();

    cube.set_source_resolution(70);
    assert_eq!(&cube.get_text().unwrap(), include_str!("../img.txt"));
}

#[test]
fn expanded_test() {
    let mut cube = Tesseract::new();
    cube.set_lang("eng").unwrap();
    cube.set_image("img.png");
    cube.set_variable("tessedit_char_blacklist", "z").unwrap();
    cube.recognize().unwrap();
    assert_eq!(&cube.get_text().unwrap(), include_str!("../img.txt"));
}
