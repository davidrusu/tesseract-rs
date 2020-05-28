//! A direct but safe wrapper for `tesseract-sys`. It should stick as close as
//! possible to the upstream API whilst avoiding unsafe behaviour.
//!
//! Are you interested in using this on its own?
//! Raise an issue, and I'll split it into its own crate.
mod pix;
mod tess_base_api;
mod tesseract_text;

pub use self::pix::Pix;
pub use self::pix::PixReadMemError;
pub use self::tess_base_api::TessBaseAPI;
pub use self::tess_base_api::TessBaseAPIGetUTF8TextError;
pub use self::tess_base_api::TessBaseAPIInitError;
pub use self::tess_base_api::TessBaseAPIRecogniseError;
pub use self::tess_base_api::TessBaseAPISetVariableError;
pub use self::tesseract_text::TesseractText;

#[test]
fn ocr_from_mem_with_ppi() {
    use std::ffi::CString;

    let pix = Pix::read_mem(include_bytes!("../../img.tiff")).unwrap();

    let mut cube = TessBaseAPI::new();
    cube.init_2(None, Some(&CString::new("eng").unwrap()))
        .unwrap();
    cube.set_image_2(&pix);

    cube.set_source_resolution(70);
    assert_eq!(
        cube.get_utf8_text().unwrap().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    );
}

#[test]
fn expanded_test() {
    use std::ffi::CString;

    let mut cube = TessBaseAPI::new();
    cube.set_variable(
        &CString::new("tessedit_char_blacklist").unwrap(),
        &CString::new("z").unwrap(),
    )
    .unwrap();
    cube.init_2(None, None).unwrap();
    let pix = Pix::read(&CString::new("../img.png").unwrap()).unwrap();
    cube.set_image_2(&pix);
    cube.recognize().unwrap();
    assert_eq!(
        cube.get_utf8_text().unwrap().as_ref().to_str(),
        Ok(include_str!("../../img.txt"))
    )
}

#[test]
fn setting_image_without_initializing_test() {
    let mut cube = TessBaseAPI::new();
    let pix = Pix::read_mem(include_bytes!("../../img.tiff")).unwrap();
    cube.set_image_2(&pix);
    assert!(cube.recognize().is_err());
    assert!(cube.get_utf8_text().is_err());
}
