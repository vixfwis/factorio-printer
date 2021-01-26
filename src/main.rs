const VERSION: &'static str = env!("CARGO_PKG_VERSION");
mod dithering;
mod blueprint;

use crate::dithering::FactorioColorMap;
use crate::blueprint::{get_icons, fserialize, schema};
use clap::{App, Arg};
use image::io::Reader as ImageReader;
use std::io::Write;
use image::{Rgba, RgbaImage};
use image::imageops::colorops::dither;
use std::ffi::OsStr;
use std::fs::File;

fn get_blueprint_simple(img: &RgbaImage, original: &RgbaImage, colors: &FactorioColorMap) -> schema::FactorioBlueprint {
    let icons = get_icons(1234);
    let mut bp = schema::FactorioBlueprint::new("test print", icons);
    for (
        (x1,y1, pix1),
        (_x2,_y2, pix2)
    ) in img.enumerate_pixels().zip(original.enumerate_pixels()) {
        if pix2.0[3] < 128 {
            continue;
        }
        match colors.get_fo(pix1) {
            Some(fo) => {
                if fo.is_tile {
                    bp.add_tile(&fo.name, x1 as i32, y1 as i32);
                }else {
                    bp.add_entity(&fo.name, x1 as i32, y1 as i32);
                }
            },
            None => {}
        }
    }
    bp
}

fn main() {
    let matches = App::new("Factorio Printer")
        .version(VERSION)
        .about("Converts images to blueprints")
        .arg(
            Arg::with_name("input_image")
                .index(1)
                .value_name("IMAGE")
                .help("Input image file")
                .required(true)
        ).get_matches();

    let input_image = std::path::Path::new(matches.value_of("input_image").unwrap());
    if !input_image.is_file() {
        eprintln!("\"{}\" is not a file", input_image.display());
        return;
    }
    let file_stem = input_image.with_extension("");
    let file_name = match file_stem.file_name().and_then(OsStr::to_str) {
        None => {
            eprintln!("error parsing \"{}\" for file name", input_image.display());
            return;
        }
        Some(e) => e
    };
    let output_image = format!("{}_converted.png", file_name);
    let output_print = format!("{}_blueprint.txt", file_name);

    let mut colors = FactorioColorMap::new();
    // refined concrete
    colors.add(Rgba::from([100, 0, 0, 255]), "refined-concrete-red", true);
    colors.add(Rgba::from([8, 97, 19, 255]), "refined-concrete-green", true);
    colors.add(Rgba::from([16, 70, 115, 255]), "refined-concrete-blue", true);
    colors.add(Rgba::from([107, 61, 16, 255]), "refined-concrete-orange", true);
    colors.add(Rgba::from([107, 85, 8, 255]), "refined-concrete-yellow", true);
    colors.add(Rgba::from([115, 49, 66, 255]), "refined-concrete-pink", true);
    colors.add(Rgba::from([58, 12, 82, 255]), "refined-concrete-purple", true);
    colors.add(Rgba::from([8, 12, 8, 255]), "refined-concrete-black", true);
    colors.add(Rgba::from([33, 12, 0, 255]), "refined-concrete-brown", true);
    colors.add(Rgba::from([33, 97, 90, 255]), "refined-concrete-cyan", true);
    colors.add(Rgba::from([67, 97, 16, 255]), "refined-concrete-acid", true);
    colors.add(Rgba::from([123, 125, 123, 255]), "refined-concrete-white", true);
    // non-modded
    colors.add(Rgba::from([47, 49, 41, 255]), "refined-concrete", true);
    colors.add(Rgba::from([115, 93, 25, 255]), "refined-hazard-concrete-left", true);
    // more tiles
    colors.add(Rgba::from([82, 81, 74, 255]), "stone-path", true);
    colors.add(Rgba::from([58, 61, 58, 255]), "concrete", true);
    colors.add(Rgba::from([181, 142, 33, 255]), "hazard-concrete-left", true);
    // entities
    colors.add(Rgba::from([0, 93, 148, 255]), "wooden-chest", false);
    colors.add(Rgba::from([206, 158, 66, 255]), "transport-belt", false);
    colors.add(Rgba::from([206, 215, 206, 255]), "stone-wall", false);

    match ImageReader::open(input_image) {
        Ok(reader) => {
            match reader.decode() {
                Ok(dyn_img) => {
                    let orig_img = dyn_img.to_rgba8();
                    let mut new_img = orig_img.clone();
                    dither(&mut new_img, &colors);
                    match new_img.save(output_image) {
                        Ok(_) => {}
                        Err(e) => { eprintln!("error writing image file: {}", e) }
                    }
                    let bp = get_blueprint_simple(&new_img, &orig_img, &colors);
                    let bpstring = fserialize(&bp).unwrap();
                    let mut file = File::create(output_print).unwrap();
                    match file.write(bpstring.as_bytes()) {
                        Ok(_) => {}
                        Err(e) => { eprintln!("error writing blueprint file: {}", e) }
                    }
                }
                Err(e) => eprintln!("error processing file \"{}\": {}", input_image.display(), e)
            }
        }
        Err(e) => eprintln!("error opening file \"{}\": {}", input_image.display(), e)
    };
}
