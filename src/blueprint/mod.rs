pub mod schema;
use schema::{FactorioSignal, FactorioIcon};
use serde_json::to_string;
use serde::Serialize;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use base64;

pub fn get_signal_by_number(value: i32) -> FactorioSignal {
    if value > 9 || value < 0 {
        panic!("get_signal_by_number: value should be between 0 and 9")
    }
    FactorioSignal { name: format!("signal-{}", value), signal_type: "virtual".to_string() }
}

pub fn get_icons(value: i32) -> Vec<FactorioIcon> {
    if value > 9999 || value < 0 {
        panic!("get_icons: value should be between 0 and 9999")
    }
    let digit0 = (value / 1000) % 10;
    let digit1 = (value / 100) % 10;
    let digit2 = (value / 10) % 10;
    let digit3 = value % 10;
    vec![
        FactorioIcon { index: 1, signal: get_signal_by_number(digit0)},
        FactorioIcon { index: 2, signal: get_signal_by_number(digit1)},
        FactorioIcon { index: 3, signal: get_signal_by_number(digit2)},
        FactorioIcon { index: 4, signal: get_signal_by_number(digit3)},
    ]
}

pub fn fserialize<T>(value: &T) -> Result<String, Box<dyn std::error::Error>>
where T: ?Sized + Serialize {
    let json_std = to_string(value)?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(json_std.as_bytes())?;
    let compr = encoder.finish()?;
    let out = base64::encode(compr);
    Ok(format!("0{}", out))
}