const VERSION: &'static str = env!("CARGO_PKG_VERSION");
mod printer;

use clap::{Arg, ArgMatches, Command, value_parser};
use std::io::{Read, Write, stdin, stdout, Cursor};
use image::{Rgba, RgbaImage};
use image::imageops::colorops::dither;
use std::fs::File;
use std::path::Path;
use image::imageops::{CatmullRom, resize};
use crate::printer::{FactorioBPStringBuilder, Tileset};

type PrinterResult<T> = Result<T, Box<dyn std::error::Error>>;

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

fn get_input_from_path(path: &str) -> PrinterResult<Box<dyn Read>> {
    if path == "-" {
        Ok(Box::new(stdin()))
    } else {
        let file = File::open(path)?;
        Ok(Box::new(file))
    }
}

fn get_output_from_path(path: &str) -> PrinterResult<Option<Box<dyn Write>>> {
    if path == "-" {
        Ok(Some(Box::new(stdout())))
    } else if path == "!" {
        Ok(None)
    } else {
        let file = File::create(path)?;
        Ok(Some(Box::new(file)))
    }
}

fn process_image(
    name: &str,
    input: Box<dyn Read>,
    out_img: Option<Box<dyn Write>>,
    out_bp: Option<Box<dyn Write>>,
    tileset: &Tileset,
    args: &ArgMatches
) -> PrinterResult<()> {
    let image_buffer = printer::read_all(input)?;
    let format = image::guess_format(&image_buffer)?;
    let mut image = image::load_from_memory_with_format(&image_buffer, format)?.to_rgba8();
    let scale = *args.get_one::<f32>("scale").expect("default scale value");
    image = scale_image(image, scale);
    // alpha channel gets overwritten by dithering, so we save a copy
    let alpha_layer = extract_alpha(&image);
    dither(&mut image, tileset);

    match out_bp {
        Some(mut writer) => {
            let alpha = *args.get_one::<u8>("alpha").expect("alpha default value");
            let split = *args.get_one::<i32>("split").expect("split default value");
            let builder =
                FactorioBPStringBuilder::new(name, &image, &alpha_layer, tileset)
                .alpha_threshold(alpha)
                .split(split);
            let export_string = match builder.factorio_serialize() {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("error exporting blueprint: {}", e);
                    std::process::exit(-1)
                }
            };
            writer.write_all(export_string.as_bytes())?;
        },
        None => {}
    }

    match out_img {
        Some(mut writer) => {
            apply_alpha(&mut image, &alpha_layer);
            let mut buf = Cursor::new(vec![]);
            image.write_to(&mut buf, format)?;
            writer.write_all(buf.get_ref())?;
        },
        None => {}
    }

    Ok(())
}

fn parse_args(args: &ArgMatches) -> PrinterResult<()> {
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
            let input = get_input_from_path(path)?;
            tileset = Tileset::read(input)?;
        }
        None => {}
    }

    match args.get_one::<String>("input_image") {
        Some(path) => {
            let input = get_input_from_path(path)?;
            let name = if Path::new(path).is_file() {
                Path::new(path)
                    .file_name()
                    .expect("valid filename")
                    .to_str()
                    .expect("valid unicode filename")
                    .to_string()
            } else {
                "Printed image".to_string()
            };
            let out_img = match args.get_one::<String>("output_image") {
                Some(path) => {
                    match get_output_from_path(path)? {
                        Some(out) => Some(out),
                        None => None,
                    }
                },
                None => None,
            };
            let out_bp = match args.get_one::<String>("output_blueprint") {
                Some(path) => {
                    match get_output_from_path(path)? {
                        Some(out) => Some(out),
                        None => None,
                    }
                },
                None => None,
            };
            process_image(&name, input, out_img, out_bp, &tileset, &args)?;
        }
        None => {}
    }

    match args.get_one::<String>("export_tileset") {
        Some(path) => {
            let output = get_output_from_path(path)?;
            match output {
                Some(output) => {
                    tileset.write(output)?;
                },
                None => {}
            }
        }
        None => {}
    }
    Ok(())
}

fn main() {
    let cmd = Command::new("Factorio Printer")
        .version(VERSION)
        .about("Factorio image (blue)printing tool\n\
        FILE inputs support '-' for stdin\n\
        FILE outputs support '-' for stdin, '!' to disable\
        ")
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
            .help("Output image")
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
    match parse_args(&args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(-1)
        }
    }
}
