use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct FactorioBlueprintInternal {
    item: String,
    label: String,
    entities: Vec<FactorioEntity>,
    tiles: Vec<FactorioTile>,
    icons: Vec<FactorioIcon>,
    #[serde(skip_serializing)]
    entity_counter: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioBlueprint {
    blueprint: FactorioBlueprintInternal
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioSignal {
    pub(crate) name: String,
    #[serde(rename="type")]
    pub(crate) signal_type: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioIcon {
    pub(crate) index: i32,  // 1-based
    pub(crate) signal: FactorioSignal
}

#[derive(Debug, Serialize, Deserialize)]
struct FactorioBookBlueprintVecElement {
    index: i32, // 0-based
    blueprint: FactorioBlueprintInternal
}

#[derive(Debug, Serialize, Deserialize)]
struct FactorioBookInternal {
    item: String,
    label: String,
    blueprints: Vec<FactorioBookBlueprintVecElement>,
    active_index: i32,  // 0-based
    version: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioBook {
    blueprint_book: FactorioBookInternal
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioEntity {
    entity_number: i32,  // 1-based
    name: String,
    position: FactorioPosition
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioTile {
    name: String,
    position: FactorioPosition
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorioPosition {
    x: i32,
    y: i32
}

impl FactorioBlueprint {
    pub fn new() -> FactorioBlueprint {
        let bp = FactorioBlueprintInternal {
            item: "blueprint".to_string(),
            label: "Blueprint".to_string(),
            entities: vec![],
            tiles: vec![],
            icons: vec![],
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

    fn digit_as_signal(&self, value: i32) -> FactorioSignal {
        FactorioSignal { name: format!("signal-{}", value), signal_type: "virtual".to_string() }
    }

    pub fn set_icons(&mut self, value: i32) {
        if value > 9999 || value < 0 {
            panic!("set_icons: value should be between 0 and 9999")
        }
        let digit0 = (value / 1000) % 10;
        let digit1 = (value / 100) % 10;
        let digit2 = (value / 10) % 10;
        let digit3 = value % 10;
        self.blueprint.icons = vec![
            FactorioIcon { index: 1, signal: self.digit_as_signal(digit0)},
            FactorioIcon { index: 2, signal: self.digit_as_signal(digit1)},
            FactorioIcon { index: 3, signal: self.digit_as_signal(digit2)},
            FactorioIcon { index: 4, signal: self.digit_as_signal(digit3)},
        ];
    }

    pub fn set_label(&mut self, label: String) {
        self.blueprint.label = label;
    }
}

impl FactorioBook {
    pub fn new() -> FactorioBook {
        let book = FactorioBookInternal {
            item: "blueprint-book".to_string(),
            label: "Book".to_string(),
            blueprints: vec![],
            active_index: 0,
            version: 0
        };
        FactorioBook{ blueprint_book: book }
    }

    pub fn add_blueprint(&mut self, bp: FactorioBlueprint) {
        self.blueprint_book.blueprints.push(
            FactorioBookBlueprintVecElement {
                index: self.blueprint_book.blueprints.len() as i32,
                blueprint: bp.blueprint
            }
        );
    }

    pub fn set_label(&mut self, label: String) {
        self.blueprint_book.label = label;
    }
}