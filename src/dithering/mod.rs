use image::Rgba;
use image::imageops::ColorMap;

const COLOR_FILTER: [i32; 3] = [11, 59, 30];

pub struct FactorioObject{
    pub(crate) name: String,
    pub(crate) is_tile: bool
}

pub struct FactorioColorMap {
    objects: Vec<(Rgba<u8>, FactorioObject)>,
}

impl FactorioColorMap {
    pub fn new() -> FactorioColorMap {
        FactorioColorMap {objects: Vec::new()}
    }

    fn find_nearest_color(&self, color: &Rgba<u8>) -> Option<Rgba<u8>> {
        let mut min_color_dist = i32::max_value();
        let mut chosen_color = None;

        for (self_color,_self_fo) in &self.objects {
            let color_dist =
                (color.0[0] as i32 - self_color.0[0] as i32)
                    * (color.0[0] as i32 - self_color.0[0] as i32) * COLOR_FILTER[0]
                    + (color.0[1] as i32 - self_color.0[1] as i32)
                    * (color.0[1] as i32 - self_color.0[1] as i32) * COLOR_FILTER[1]
                    + (color.0[2] as i32 - self_color.0[2] as i32)
                    * (color.0[2] as i32 - self_color.0[2] as i32) * COLOR_FILTER[2];
            if color_dist < min_color_dist {
                min_color_dist = color_dist;
                chosen_color = Some(*self_color);
            }
        }

        chosen_color
    }

    pub fn get_fo(&self, color: &Rgba<u8>) -> Option<FactorioObject> {
        for (k,v) in &self.objects {
            if *k == *color {
                return Some(FactorioObject{ name: v.name.clone(), is_tile: v.is_tile });
            }
        }
        None
    }

    pub fn add(&mut self, color: Rgba<u8>, name: &str, is_tile: bool) {
        self.objects.push((color, FactorioObject{name: String::from(name), is_tile}));
    }
}

impl ColorMap for FactorioColorMap {
    type Color = Rgba<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        for (i, value) in self.objects.iter().enumerate() {
            if *color == value.0 {
                return i;
            }
        }
        return usize::MAX;
    }

    fn map_color(&self, color: &mut Self::Color) {
        let new_color = self.find_nearest_color(color).expect("Nearest color not found smh. Got any colors?");
        *color = new_color;
    }
}
