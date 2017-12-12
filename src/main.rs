extern crate image;
extern crate palette;

use std::env;
use std::process;
use std::fs::File;
use std::path::PathBuf;

mod options;
mod sorters;

#[cfg(test)]
mod tests;


fn get_format(path: &PathBuf) -> Result<image::ImageFormat, String> {
    let ext = path.extension().and_then(|s| s.to_str())
                  .map_or("".to_string(), |s| s.to_ascii_lowercase());
    match &ext[..] {
        "jpg" |
        "jpeg" => Ok(image::ImageFormat::JPEG),
        "png"  => Ok(image::ImageFormat::PNG),
        "gif"  => Ok(image::ImageFormat::GIF),
        "webp" => Ok(image::ImageFormat::WEBP),
        "tif" |
        "tiff" => Ok(image::ImageFormat::TIFF),
        "tga" => Ok(image::ImageFormat::TGA),
        "bmp" => Ok(image::ImageFormat::BMP),
        "ico" => Ok(image::ImageFormat::ICO),
        "hdr" => Ok(image::ImageFormat::HDR),
        "pnm" => Ok(image::ImageFormat::PNM),
        format => Err(format!(
            "Image format image/{} is not supported.", format)),
    }
}

fn sort_pixels(img: &mut image::DynamicImage, mode: &options::Mode)
        -> image::RgbaImage {
    let key_fn = match *mode {
        options::Mode::Red => sorters::get_red,
        options::Mode::Green => sorters::get_green,
        options::Mode::Blue => sorters::get_blue,
        options::Mode::Alpha => sorters::get_alpha,
        options::Mode::Hue => sorters::get_hue,
        options::Mode::Saturation => sorters::get_sat,
        options::Mode::Lightness => sorters::get_lig,
    };

    use image::GenericImage;

    // cast to sted::Vec<pixels> for sorting
    let bytes_per_pixel: usize = std::mem::size_of::<image::Rgba<u8>>();
    let mut v_from_raw = unsafe {
        let mut pixel_buf = img.to_rgba().into_raw();
        Vec::from_raw_parts(pixel_buf.as_mut_ptr() as *mut image::Rgba<u8>,
                            pixel_buf.len()/bytes_per_pixel,
                            pixel_buf.capacity()/bytes_per_pixel)
    };
    v_from_raw.sort_unstable_by_key(key_fn);
    // cast back to image::DynamicImage
    let sorted_pixels = unsafe {
        Vec::from_raw_parts(v_from_raw.as_mut_ptr() as *mut u8,
                            v_from_raw.len()*bytes_per_pixel,
                            v_from_raw.capacity()*bytes_per_pixel)
    };
    std::mem::forget(v_from_raw);
    image::ImageBuffer::from_raw(img.width(), img.height(), sorted_pixels).expect("Could not create image after sorting")
}

fn main() {
    let opts = options::parse(env::args_os().collect());

    let mut img = match image::open(&opts.inpath) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let buf = sort_pixels(&mut img, &opts.mode);

    let format = match get_format(&opts.outpath) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let mut fout = match File::create(&opts.outpath) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    match image::ImageRgba8(buf).save(&mut fout, format) {
        Ok(_) => process::exit(0),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };
}
