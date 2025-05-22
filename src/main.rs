use chrono::prelude::*;
use image::RgbaImage;
use leptess::tesseract;
use opencv::{
    core::{self, VecN},
    highgui, imgcodecs,
    imgproc::{self, LINE_8},
    prelude::*,
};
use std::{thread, time::Duration};
use std::{ffi::CString, fs, path::Path};
use types::tag::{self, image_to_tags};
use xcap::image;
use xcap::Window;
mod types;

static RECRUITMENT_ROI_VERTICAL: (f64, f64) = (
    0.45, // ignore top 45%
    0.30, // ignore bottom 30%
);
static RECRUITMENT_ROI_HORIZONTAL: (f64, f64) = (
    0.3, // ignore left 30%
    0.3, // ignore right 30%
);

#[allow(dead_code)]
fn draw_boxes(
    mut image: &mut Mat,
    recs: &Vec<core::Rect>,
    texts: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(recs.len() == texts.len());
    for i in 0..texts.len() {
        imgproc::rectangle(
            &mut image,
            *recs.get(i).unwrap(),
            core::VecN([0.0, 255.0, 0.0, 255.0]),
            2,
            LINE_8,
            0,
        )?;
        imgproc::put_text(
            &mut image,
            texts.get(i).unwrap(),
            recs.get(i).unwrap().tl() + core::Point::new(0, -5),
            0,
            0.6,
            VecN([0.0, 255.0, 0.0, 0.0]),
            2,
            LINE_8,
            false,
        )?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tess: tesseract::TessApi = tesseract::TessApi::new(Some("/usr/share/tessdata"), "eng")?; //Tesseract::new(Some("path/to/tessdata"), Some("eng")).unwrap();
    let key_cstr: CString = CString::new("tessedit_char_whitelist").expect("CString::new failed");
    let value_cstr: CString = CString::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-")
        .expect("CString::new failed");
    tess.raw.set_variable(&key_cstr, &value_cstr)?;

    let windows: Vec<Window> = Window::all().unwrap();
    let window: &Window = windows
        .iter()
        .filter(|w| {
            if w.is_minimized().unwrap() || !w.title().unwrap().contains("Arknights") {
                false
            } else {
                true
            }
        })
        .collect::<Vec<&Window>>()
        .get(0)
        .unwrap();

    loop {
        let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = window.capture_image().unwrap();
        let mut converted: Mat = Mat::default();
        imgproc::cvt_color_def(&into_mat(&image), &mut converted, imgproc::COLOR_RGBA2BGR)?;

        let r: core::Rect_<i32> = core::Rect::new(
            (converted.cols() as f64 * 0.3) as i32,
            (converted.rows() as f64 * RECRUITMENT_ROI_VERTICAL.0) as i32,
            (converted.cols() as f64 * 0.35) as i32,
            (converted.rows() as f64 * 0.25) as i32,
        );

        let cropped: Mat = Mat::roi(&converted, r).unwrap().try_clone().unwrap();
        let mut tags: Vec<tag::Tag> = image_to_tags(&cropped, &mut tess)?;
        tags.sort_by(|t1, t2| {
            t1.tag_type().to_string().cmp(&t2.tag_type().to_string())
        });

        for tag in tags {
            println!("tag: [{:15}] / selected: [{:5}]", tag.tag_type().to_string().as_str(), tag.selected());
        }
        print!("{esc}c", esc = 27 as char);
        thread::sleep(Duration::new(1, 0));
    }
}

fn into_mat(image: &RgbaImage) -> Mat {
    unsafe {
        Mat::new_rows_cols_with_data_unsafe_def(
            image.height() as i32,
            image.width() as i32,
            opencv::core::CV_8UC4,
            image.as_raw().clone().as_mut_ptr() as *mut _,
        )
        .unwrap()
    }
}

#[test]
fn main_test() -> Result<(), Box<dyn std::error::Error>> {
    let path: &Path = Path::new("images/test/");
    let filenames: Vec<_> = fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let entry: fs::DirEntry = entry.unwrap();
            let file_type: fs::FileType = entry.file_type().unwrap();
            let file_name: std::ffi::OsString = entry.file_name();
            let file_str: std::borrow::Cow<'_, str> = file_name.to_string_lossy();
            if file_type.is_file() && (file_str.ends_with(".jpg") || file_str.ends_with(".png")) {
                Some(file_str.to_string())
            } else {
                None
            }
        })
        .collect();

    let mut tess: tesseract::TessApi = tesseract::TessApi::new(Some("/usr/share/tessdata"), "eng")?; //Tesseract::new(Some("path/to/tessdata"), Some("eng")).unwrap();
    let key_cstr: CString = CString::new("tessedit_char_whitelist").expect("CString::new failed");
    let value_cstr: CString = CString::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-")
        .expect("CString::new failed");
    tess.raw.set_variable(&key_cstr, &value_cstr)?;

    let start: DateTime<Utc> = Utc::now();
    let mut images: Vec<Mat> = vec![];
    for file in filenames {
        // println!("{}", file);
        let image: Mat = imgcodecs::imread(
            format!("images/test/{}", file).as_str(),
            imgcodecs::IMREAD_COLOR_BGR,
        )?;
        let r = core::Rect::new(
            (image.cols() as f64 * 0.3) as i32,
            (image.rows() as f64 * ROI_VERTICAL.0) as i32,
            (image.cols() as f64 * 0.35) as i32,
            (image.rows() as f64 * 0.25) as i32,
        );
        let cropped: Mat = Mat::roi(&image, r).unwrap().try_clone().unwrap();
        images.push(cropped);
    }

    let mut rec_tags: Vec<Vec<tag::Tag>> = vec![];
    for image in &images {
        let tags: Vec<tag::Tag> = image_to_tags(&image, &mut tess)?;
        rec_tags.push(tags);
        // println!("{:?}", texts);

        // highgui::named_window("Display Window", highgui::WINDOW_AUTOSIZE)?;
        // highgui::imshow("Display Window", &gray)?;
        // highgui::wait_key(0)?;
    }
    let end: DateTime<Utc> = Utc::now();
    let total = (end - start).as_seconds_f64();
    let avg = total / rec_tags.len() as f64;
    println!("Total: {} / Avg: {} / FPS: {}", total, avg, 1.0 / avg);
    Ok(())
}
