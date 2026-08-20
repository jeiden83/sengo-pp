use napi::bindgen_prelude::*;
use napi::Env;
use rosu_mods::{GameModsIntermode, GameModsLegacy};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ParsedModsInfo {
    pub mods: GameModsIntermode,
    pub clock_rate: Option<f64>,
    pub ar: Option<f64>,
    pub cs: Option<f64>,
    pub hp: Option<f64>,
    pub od: Option<f64>,
    pub has_hidden: bool,
    pub has_hardrock: bool,
    pub has_easy: bool,
    pub has_flashlight: bool,
}

pub fn parse_mods_full(env: &Env, val: Unknown) -> Result<ParsedModsInfo> {
    let vt = val.get_type()?;
    let mut info = ParsedModsInfo::default();

    match vt {
        ValueType::Undefined | ValueType::Null => Ok(info),
        ValueType::Number => {
            let bits = val.coerce_to_number()?.get_uint32()?;
            info.mods = GameModsLegacy::from_bits(bits).into();
            finalize_mods(&mut info);
            Ok(info)
        }
        ValueType::String => {
            let s = val.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
            if s.is_empty() || s.eq_ignore_ascii_case("NM") {
                return Ok(info);
            }
            info.mods = s
                .parse::<GameModsIntermode>()
                .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to parse mods string '{s}': {e}")))?;
            finalize_mods(&mut info);
            Ok(info)
        }
        ValueType::Object => {
            let json_val: Value = env.from_js_value(val)?;
            match json_val {
                Value::Array(arr) => {
                    let mut mods_str = String::new();
                    for item in &arr {
                        match item {
                            Value::String(s) => {
                                if !s.eq_ignore_ascii_case("NM") {
                                    mods_str.push_str(s);
                                }
                            }
                            Value::Object(obj) => {
                                if let Some(acronym) = obj.get("acronym").and_then(|v| v.as_str()) {
                                    if !acronym.eq_ignore_ascii_case("NM") {
                                        mods_str.push_str(acronym);
                                    }
                                }
                                if let Some(settings) = obj.get("settings").and_then(|v| v.as_object()) {
                                    if let Some(sc) = settings.get("speed_change").and_then(|v| v.as_f64()) {
                                        info.clock_rate = Some(sc);
                                    }
                                    if let Some(ar) = settings.get("approach_rate").and_then(|v| v.as_f64()).or_else(|| settings.get("ar").and_then(|v| v.as_f64())) {
                                        info.ar = Some(ar);
                                    }
                                    if let Some(cs) = settings.get("circle_size").and_then(|v| v.as_f64()).or_else(|| settings.get("cs").and_then(|v| v.as_f64())) {
                                        info.cs = Some(cs);
                                    }
                                    if let Some(hp) = settings.get("drain_rate").and_then(|v| v.as_f64()).or_else(|| settings.get("hp").and_then(|v| v.as_f64())) {
                                        info.hp = Some(hp);
                                    }
                                    if let Some(od) = settings.get("overall_difficulty").and_then(|v| v.as_f64()).or_else(|| settings.get("od").and_then(|v| v.as_f64())) {
                                        info.od = Some(od);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if !mods_str.is_empty() {
                        info.mods = mods_str
                            .parse::<GameModsIntermode>()
                            .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to parse mods array: {e}")))?;
                    }
                    finalize_mods(&mut info);
                    Ok(info)
                }
                Value::Object(obj) => {
                    if let Some(acronym) = obj.get("acronym").and_then(|v| v.as_str()) {
                        info.mods = acronym
                            .parse::<GameModsIntermode>()
                            .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to parse mod acronym: {e}")))?;
                    }
                    if let Some(settings) = obj.get("settings").and_then(|v| v.as_object()) {
                        if let Some(sc) = settings.get("speed_change").and_then(|v| v.as_f64()) {
                            info.clock_rate = Some(sc);
                        }
                        if let Some(ar) = settings.get("approach_rate").and_then(|v| v.as_f64()).or_else(|| settings.get("ar").and_then(|v| v.as_f64())) {
                            info.ar = Some(ar);
                        }
                        if let Some(cs) = settings.get("circle_size").and_then(|v| v.as_f64()).or_else(|| settings.get("cs").and_then(|v| v.as_f64())) {
                            info.cs = Some(cs);
                        }
                        if let Some(hp) = settings.get("drain_rate").and_then(|v| v.as_f64()).or_else(|| settings.get("hp").and_then(|v| v.as_f64())) {
                            info.hp = Some(hp);
                        }
                        if let Some(od) = settings.get("overall_difficulty").and_then(|v| v.as_f64()).or_else(|| settings.get("od").and_then(|v| v.as_f64())) {
                            info.od = Some(od);
                        }
                    }
                    finalize_mods(&mut info);
                    Ok(info)
                }
                _ => {
                    finalize_mods(&mut info);
                    Ok(info)
                }
            }
        }
        _ => Ok(info),
    }
}

fn finalize_mods(info: &mut ParsedModsInfo) {
    let bits = info.mods.bits();
    if bits & 8 != 0 {
        info.has_hidden = true;
    }
    if bits & 16 != 0 {
        info.has_hardrock = true;
    }
    if bits & 2 != 0 {
        info.has_easy = true;
    }
    if bits & 1024 != 0 {
        info.has_flashlight = true;
    }
    if info.clock_rate.is_none() {
        if bits & 64 != 0 || bits & 512 != 0 {
            info.clock_rate = Some(1.5);
        } else if bits & 256 != 0 {
            info.clock_rate = Some(0.75);
        }
    }
}

pub fn parse_mods(env: &Env, val: Unknown) -> Result<GameModsIntermode> {
    let info = parse_mods_full(env, val)?;
    Ok(info.mods)
}
