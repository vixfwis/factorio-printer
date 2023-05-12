const VERSION: &'static str = env!("CARGO_PKG_VERSION");
mod printer;

use clap::{Arg, ArgMatches, Command, value_parser};
use image::io::Reader as ImageReader;
use std::io::Write;
use image::{ImageFormat, Rgba, RgbaImage};
use image::imageops::colorops::dither;
use std::fs::File;
use std::path::Path;
use image::imageops::{CatmullRom, resize};
use crate::printer::{FactorioBPStringBuilder, Tileset};

fn extract_alpha(image: &RgbaImage) -> RgbaImage {
    let (dim_x, dim_y) = image.dimensions();
    let mut alpha_layer = RgbaImage::from_pixel(
        dim_x,
        dim_y,
        Rgba::from([0; 4])
    );
    for (x,y,pix) in image.enumerate_pixels() {
        alpha_layer.put_pixel(x, y, Rgba::from([0, 0, 0, pix.0[3]]));
    }
    alpha_layer
}

fn apply_alpha(image: &mut RgbaImage, alpha: &RgbaImage) {
    for (pix, alpha) in image.pixels_mut().zip(alpha.pixels()) {
        pix.0[3] = alpha.0[3];
    }
}

fn scale_image(mut image: RgbaImage, scale: f32) -> RgbaImage {
    if scale != 1.0f32 {
        let (dim_x, dim_y) = image.dimensions();
        let scaled_x = dim_x as f32 * scale;
        let scaled_y = dim_y as f32 * scale;
        image = resize(&mut image, scaled_x.round() as u32, scaled_y.round() as u32, CatmullRom);
    }
    image
}

fn process_image(input_image: &str, tileset: &Tileset, args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let scale = *args.get_one::<f32>("scale").expect("default scale value");
    let mut image = scale_image(ImageReader::open(input_image)?.decode()?.into_rgba8(), scale);
    // alpha channel gets overwritten by dithering, so we save a copy
    let alpha_layer = extract_alpha(&image);
    dither(&mut image, tileset);
    match args.get_one::<String>("output_image") {
        Some(path) => {
            apply_alpha(&mut image, &alpha_layer);
            match image.save_with_format(path, ImageFormat::Png) {
                Ok(_) => {}
                Err(e) => { eprintln!("error writing image file: {}", e) }
            }
        }
        None => {}
    }

    let file_name = Path::new(input_image)
        .file_name()
        .expect("there is no filename") // image would've failed earlier if None
        .to_str()
        .expect("valid unicode name");
    let export_string =
        match FactorioBPStringBuilder::new(file_name, &image, &alpha_layer, tileset)
            .alpha_threshold(*args.get_one::<u8>("alpha").expect("alpha default value"))
            .split(*args.get_one::<i32>("split").expect("split default value"))
            .factorio_serialize() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("error exporting blueprint: {}", e);
                std::process::exit(-1)
            }
        };

    match args.get_one::<String>("output_blueprint") {
        Some(path) => {
            let mut file = File::create(path).unwrap();
            match file.write(export_string.as_bytes()) {
                Ok(_) => {}
                Err(e) => { eprintln!("error writing blueprint file: {}", e) }
            }
        }
        None => {}
    }
    Ok(())
}

fn main() {
    let cmd = Command::new("Factorio Printer")
        .version(VERSION)
        .about("Factorio image (blue)printing tool")
        .arg_required_else_help(true)
        .disable_version_flag(true)
        .arg(Arg::new("input_image")
            .index(1)
            .value_name("FILE")
            .help("Input image file")
            .required(false))
        .arg(Arg::new("output_image")
            .short('o')
            .value_name("FILE")
            .help("Output image in PNG format")
            .default_value("output.png"))
        .arg(Arg::new("output_blueprint")
             .short('b')
             .value_name("FILE")
             .help("Output blueprint")
             .default_value("blueprint.txt"))
        .arg(Arg::new("scale")
            .short('s')
            .long("scale")
            .help("Scaling factor")
            .value_parser(value_parser!(f32))
            .default_value("1.0"))
        .arg(Arg::new("preset")
            .short('p')
            .long("preset")
            .value_parser(["base", "colorcoding"])
            .default_value("colorcoding")
            .help("Built-in tilesets"))
        .arg(Arg::new("tileset")
            .short('t')
            .long("tileset")
            .value_name("FILE")
            .help("Alternative tileset"))
        .arg(Arg::new("export_tileset")
            .long("export-tileset")
            .value_name("FILE")
            .help("Export current tileset in CSV format"))
        .arg(Arg::new("alpha")
            .long("alpha")
            .value_name("VALUE")
            .value_parser(clap::builder::RangedU64ValueParser::<u8>::new().range(1..256))
            .help("Pixels with alpha channel less that <VALUE> are skipped")
            .default_value("128"))
        .arg(Arg::new("split")
            .long("split")
            .value_name("SIDE")
            .value_parser(clap::builder::RangedI64ValueParser::<i32>::new().range(0..10000))
            .help("Split blueprint into squares of <SIDE>^2 size. 0 means no splitting")
            .default_value("0"));
    let args = cmd.get_matches();

    let mut tileset = Tileset::preset_color_coding();

    match args.get_one::<String>("preset") {
        Some(preset) => {
            if preset == "base" {
                tileset = Tileset::preset_base_game();
            }
        }
        None => {}
    }

    match args.get_one::<String>("tileset") {
        Some(path) => {
            tileset = match Tileset::from_file(path) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("error saving tileset: {}", e);
                    std::process::exit(-1)
                }
            }
        }
        None => {}
    }

    match args.get_one::<String>("input_image") {
        Some(path) => {
            match process_image(path, &tileset, &args) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error in process_image: {}", e);
                    std::process::exit(-1)
                }
            }
        }
        None => {}
    }

    match args.get_one::<String>("export_tileset") {
        Some(path) => {
            match tileset.to_file(path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error saving tileset: {}", e);
                    std::process::exit(-1)
                }
            }
        }
        None => {}
    }
}
