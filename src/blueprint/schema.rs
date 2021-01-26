use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FactorioBlueprintInternal {
    item: String,
    label: String,
    entities: Vec<FactorioEntity>,
    tiles: Vec<FactorioTile>,
    icons: Vec<FactorioIcon>,
    #[serde(skip_serializing)]
    entity_counter: i32
}

#[derive(Serialize, Deserialize)]
pub struct FactorioBlueprint {
    blueprint: FactorioBlueprintInternal
}

#[derive(Serialize, Deserialize)]
pub struct FactorioSignal {
    pub(crate) name: String,
    #[serde(rename="type")]
    pub(crate) signal_type: String
}

#[derive(Serialize, Deserialize)]
pub struct FactorioIcon {
    pub(crate) index: i32,  // 1-based
    pub(crate) signal: FactorioSignal
}

#[derive(Serialize, Deserialize)]
pub struct FactorioBookInternal {
    item: String,
    label: String,
    blueprints: Vec<FactorioBlueprintInternal>,
    active_index: i32,  // 0-based
    version: i64
}

#[derive(Serialize, Deserialize)]
pub struct FactorioBook {
    blueprint_book: FactorioBookInternal
}

#[derive(Serialize, Deserialize)]
pub struct FactorioEntity {
    entity_number: i32,  // 1-based
    name: String,
    position: FactorioPosition
}

#[derive(Serialize, Deserialize)]
pub struct FactorioTile {
    name: String,
    position: FactorioPosition
}

#[derive(Serialize, Deserialize)]
pub struct FactorioPosition {
    x: i32,
    y: i32
}

impl FactorioBlueprint {
    pub fn new(label: &str, icons: Vec<FactorioIcon>) -> FactorioBlueprint {
        let bp = FactorioBlueprintInternal {
            item: "blueprint".to_string(),
            label: label.to_string(),
            entities: vec![],
            tiles: vec![],
            icons,
            entity_counter: 1
        };
        FactorioBlueprint{ blueprint: bp }
    }

    pub fn add_entity(&mut self, name: &str, x: i32, y: i32) {
        self.blueprint.entities.push(FactorioEntity{
            entity_number: self.blueprint.entity_counter,
            name: name.to_string(),
            position: FactorioPosition {x, y}
        });
        self.blueprint.entity_counter += 1;
    }

    pub fn add_tile(&mut self, name: &str, x: i32, y: i32) {
        self.blueprint.tiles.push(FactorioTile{
            name: name.to_string(),
            position: FactorioPosition {x, y}
        });
    }
}

#[allow(dead_code)]
impl FactorioBook {
    pub fn new(label: &str) -> FactorioBook {
        let book = FactorioBookInternal {
            item: "blueprint-book".to_string(),
            label: label.to_string(),
            blueprints: vec![],
            active_index: 0,
            version: 0
        };
        FactorioBook{ blueprint_book: book }
    }

    pub fn add_blueprint(&mut self, bp: FactorioBlueprintInternal) {
        self.blueprint_book.blueprints.push(bp);
    }
}