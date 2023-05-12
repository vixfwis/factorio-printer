use image::{Pixel, Rgb, Rgba, RgbaImage};
use image::imageops::ColorMap;
use serde::{Deserialize, Serialize};
use serde_json;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use base64::engine::general_purpose::STANDARD as B64Engine;
use std::io::Write;
use base64::Engine;
use crate::printer::schema::{FactorioBlueprint, FactorioBook};

mod schema;

// const COLOR_FILTER: [i32; 3] = [11, 59, 30];
const COLOR_FILTER: [i32; 3] = [1, 1, 1];

const TILESET_BASE: [(u8, u8, u8, &str, bool); 8] = [
    (47, 49, 41, "refined-concrete", true),
    (115, 93, 25, "refined-hazard-concrete-left", true),
    (82, 81, 74, "stone-path", true),
    (58, 61, 58, "concrete", true),
    (181, 142, 33, "hazard-concrete-left", true),
    (0, 93, 148, "wooden-chest", false),
    (206, 158, 66, "transport-belt", false),
    (206, 215, 206, "stone-wall", false),
];

const TILESET_COLOR_CODING: [(u8, u8, u8, &str, bool); 20] = [
    (47, 49, 41, "refined-concrete", true),
    (115, 93, 25, "refined-hazard-concrete-left", true),
    (82, 81, 74, "stone-path", true),
    (58, 61, 58, "concrete", true),
    (181, 142, 33, "hazard-concrete-left", true),
    (0, 93, 148, "wooden-chest", false),
    (206, 158, 66, "transport-belt", false),
    (206, 215, 206, "stone-wall", false),

    (100, 0, 0, "refined-concrete-red", true),
    (8, 97, 19, "refined-concrete-green", true),
    (16, 70, 115, "refined-concrete-blue", true),
    (107, 61, 16, "refined-concrete-orange", true),
    (107, 85, 8, "refined-concrete-yellow", true),
    (115, 49, 66, "refined-concrete-pink", true),
    (58, 12, 82, "refined-concrete-purple", true),
    (8, 12, 8, "refined-concrete-black", true),
    (33, 12, 0, "refined-concrete-brown", true),
    (33, 97, 90, "refined-concrete-cyan", true),
    (67, 97, 16, "refined-concrete-acid", true),
    (123, 125, 123, "refined-concrete-white", true),
];

#[derive(Serialize, Deserialize)]
struct Tile {
    red: u8,
    green: u8,
    blue: u8,
    name: String,
    is_tile: bool
}

impl Tile {
    fn new(red: u8, green: u8, blue: u8, name: &str, is_tile: bool) -> Self {
        Tile {
            red,
            green,
            blue,
            name: name.to_string(),
            is_tile,
        }
    }

    fn rgb(&self) -> Rgb<u8> {
        Rgb::from([self.red, self.green, self.blue])
    }

    fn rgba(&self) -> Rgba<u8> {
        Rgba::from([self.red, self.green, self.blue, 255])
    }
}

pub struct Tileset {
    tiles: Vec<Tile>
}

impl Tileset {
    fn from_const(obj: &[(u8, u8, u8, &str, bool)]) -> Self {
        let mut tiles = vec![];
        for (r, g, b, name, is_tile) in obj {
            tiles.push(Tile::new(*r,*g,*b,name,*is_tile))
        }
        Tileset { tiles }
    }

    pub fn preset_base_game() -> Self {
        Self::from_const(&TILESET_BASE)
    }

    pub fn preset_color_coding() -> Self {
        Self::from_const(&TILESET_COLOR_CODING)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_path(path)?;
        for tile in &self.tiles {
            writer.serialize(tile)?;
        }
        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut tiles = vec![];
        let mut reader = csv::Reader::from_path(path)?;
        for row in reader.deserialize() {
            let tile: Tile = row?;
            tiles.push(tile);
        }
        Ok(Tileset { tiles })
    }

    fn get_matching_tile(&self, color: &Rgb<u8>) -> &Tile {
        self.tiles.iter()
            .find(|tile| tile.rgb() == *color)
            .expect("No matching tiles. Did you forget the dithering?")
    }

    fn find_closest_color(&self, color: &Rgb<u8>) -> Rgb<u8> {
        let index = self.tiles.iter().enumerate().map(|(idx, tile)|
            (idx,
            (color.0[0] as i32 - tile.red as i32)
                * (color.0[0] as i32 - tile.red as i32) * COLOR_FILTER[0]
                + (color.0[1] as i32 - tile.green as i32)
                * (color.0[1] as i32 - tile.green as i32) * COLOR_FILTER[1]
                + (color.0[2] as i32 - tile.blue as i32)
                * (color.0[2] as i32 - tile.blue as i32) * COLOR_FILTER[2])
        ).min_by(|(_, a), (_, b)|
            a.cmp(b)
        ).map(|(idx, _)| idx).expect("Tileset must not be empty");
        self.tiles[index].rgb()
    }
}

impl ColorMap for Tileset {
    type Color = Rgba<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        for (i, tile) in self.tiles.iter().enumerate() {
            if *color == tile.rgba() {
                return i;
            }
        }
        return usize::MAX;
    }

    fn map_color(&self, color: &mut Self::Color) {
        *color = self.find_closest_color(&color.to_rgb()).to_rgba();
    }
}

pub struct FactorioBPStringBuilder<'a> {
    label: String,
    image: &'a RgbaImage,
    alpha: &'a RgbaImage,
    tileset: &'a Tileset,

    alpha_threshold: u8,
    split: i32,
    split_count_x: i32,
    split_count_y: i32,
}

impl FactorioBPStringBuilder<'_> {
    pub fn new<'a>(
        label: &str,
        image: &'a RgbaImage,
        alpha: &'a RgbaImage,
        tileset: &'a Tileset
    ) -> FactorioBPStringBuilder<'a> {
        FactorioBPStringBuilder {
            label: label.to_string(),
            image,
            alpha,
            tileset,
            alpha_threshold: 128,
            split: 0,
            split_count_x: 1,
            split_count_y: 1,
        }
    }

    pub fn alpha_threshold(mut self, value: u8) -> Self {
        self.alpha_threshold = value;
        self
    }

    pub fn split(mut self, value: i32) -> Self {
        self.split = value;
        if self.split <= 0 {
            self.split_count_x = 1;
            self.split_count_y = 1;
        } else {
            let (width, height) = self.image.dimensions();
            self.split_count_x = (width as f32 / self.split as f32).ceil() as i32;
            self.split_count_y = (height as f32 / self.split as f32).ceil() as i32;
        }
        self
    }

    fn get_bp_split_coords(&self, x: i32, y: i32) -> (usize, i32, i32) {
        if self.split == 0 {
            (0, x, y)
        } else {
            let split_x = x % self.split;
            let split_y = y % self.split;
            let vec_idx_x = x / self.split;
            let vec_idx_y = y / self.split;
            let vec_index = vec_idx_x + vec_idx_y * self.split_count_x;
            (vec_index as usize, split_x, split_y)
        }
    }

    fn get_total_bp_count(&self) -> i32 {
        self.split_count_x * self.split_count_y
    }

    fn make_blueprints(&self) -> Vec<FactorioBlueprint> {
        let mut prints = vec![];
        let mut icons = true;
        for i in 0..self.get_total_bp_count() {
            prints.push(FactorioBlueprint::new());
            let x = i % self.split_count_x;
            let y = i / self.split_count_x;
            prints[i as usize].set_label(format!("{}: x: {} y: {}", &self.label, x, y));
            if icons {
                if x > 99 || y > 99 {
                    icons = false;
                } else {
                    prints[i as usize].set_icons(x * 100 + y);
                }
            }
        }
        if !icons {
            eprintln!("warning: resulting split side count >99, icons will be set to 0");
            for print in &mut prints {
                print.set_icons(0);
            }
        }
        for (
            (x1, y1, pix1),
            (_x2, _y2, pix2)
        ) in self.image.enumerate_pixels().zip(self.alpha.enumerate_pixels()) {
            if pix2.0[3] < self.alpha_threshold {
                continue;
            }
            let tile = self.tileset.get_matching_tile(&pix1.to_rgb());
            let (idx, x, y) = self.get_bp_split_coords(x1 as i32, y1 as i32);
            // println!("pixel {},{} goes to idx({}) x({}) y({})", x1,y1, idx,x,y);
            if tile.is_tile {
                prints[idx].add_tile(&tile.name, x, y);
            } else {
                prints[idx].add_entity(&tile.name, x, y);
            }
        }
        prints
    }

    fn make_book(&self) -> FactorioBook {
        let mut book = FactorioBook::new();
        book.set_label(format!("{}", &self.label));
        for bp in self.make_blueprints() {
            book.add_blueprint(bp)
        }
        book
    }

    pub fn factorio_serialize(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json_std = if self.split_count_x == 1 {
            serde_json::to_string(&self.make_blueprints()[0])?
        } else {
            serde_json::to_string(&self.make_book())?
        };
        // println!("{}", &json_std);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_std.as_bytes())?;
        let compr = encoder.finish()?;
        let out = B64Engine.encode(compr);
        Ok(format!("0{}", out))
    }
}




