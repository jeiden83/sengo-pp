use napi::bindgen_prelude::*;
use napi_derive::napi;
use rosu_pp::model::beatmap::BeatmapAttributesBuilder;
use rosu_pp::GameMods;
use crate::enums::GameMode;
use crate::mods::parse_mods_full;

#[napi(object, js_name = "BeatmapAttributes")]
#[derive(Debug, Clone)]
pub struct JsBeatmapAttributes {
    pub ar: f64,
    pub cs: f64,
    pub hp: f64,
    pub od: f64,
    pub clock_rate: f64,
}

#[napi(object)]
pub struct BeatmapAttributesArgs {
    pub mode: Option<GameMode>,
    pub mods: Option<Unknown>,
    pub clock_rate: Option<f64>,
    pub ar: Option<f64>,
    pub cs: Option<f64>,
    pub hp: Option<f64>,
    pub od: Option<f64>,
    pub is_convert: Option<bool>,
}

#[napi(js_name = "BeatmapAttributesBuilder")]
pub struct JsBeatmapAttributesBuilder {
    pub(crate) builder: BeatmapAttributesBuilder,
    pub(crate) mode: Option<GameMode>,
    pub(crate) is_convert: bool,
}

fn calculate_ar(ar: f64, clock_rate: f64) -> f64 {
    if clock_rate == 1.0 {
        return ar;
    }
    let preempt = if ar <= 5.0 {
        1800.0 - ar * 120.0
    } else {
        1200.0 - (ar - 5.0) * 150.0
    };
    let modded_preempt = preempt / clock_rate;
    if modded_preempt > 1200.0 {
        (1800.0 - modded_preempt) / 120.0
    } else {
        5.0 + (1200.0 - modded_preempt) / 150.0
    }
}

fn calculate_od(od: f64, clock_rate: f64) -> f64 {
    if clock_rate == 1.0 {
        return od;
    }
    let hit_window = 80.0 - 6.0 * od;
    let modded_hit_window = hit_window / clock_rate;
    (80.0 - modded_hit_window) / 6.0
}

fn calculate_taiko_od(od: f64, clock_rate: f64) -> f64 {
    if clock_rate == 1.0 {
        return od;
    }
    let hit_window = 50.0 - 3.0 * od;
    let modded = hit_window / clock_rate;
    (50.0 - modded) / 3.0
}

#[napi]
impl JsBeatmapAttributesBuilder {
    #[napi(constructor)]
    pub fn new(env: Env, args: Option<BeatmapAttributesArgs>) -> Result<Self> {
        let mut builder = BeatmapAttributesBuilder::new();
        let mut mode = None;
        let mut is_convert = false;

        if let Some(args) = args {
            if let Some(m) = args.mode {
                mode = Some(m);
            }
            if let Some(c) = args.is_convert {
                is_convert = c;
            }
            if let Some(m) = mode {
                builder.mode(m.into(), is_convert);
            }
            if let Some(mods_unknown) = args.mods {
                let info = parse_mods_full(&env, mods_unknown)?;
                let game_mods = GameMods::from(info.mods);
                builder.mods(game_mods);
                if let Some(cr) = info.clock_rate {
                    builder.clock_rate(cr);
                }
                if let Some(ar) = info.ar {
                    builder.ar(ar as f32, false);
                }
                if let Some(cs) = info.cs {
                    builder.cs(cs as f32, false);
                }
                if let Some(hp) = info.hp {
                    builder.hp(hp as f32, false);
                }
                if let Some(od) = info.od {
                    builder.od(od as f32, false);
                }
            }
            if let Some(cr) = args.clock_rate {
                builder.clock_rate(cr);
            }
            if let Some(ar) = args.ar {
                builder.ar(ar as f32, false);
            }
            if let Some(cs) = args.cs {
                builder.cs(cs as f32, false);
            }
            if let Some(hp) = args.hp {
                builder.hp(hp as f32, false);
            }
            if let Some(od) = args.od {
                builder.od(od as f32, false);
            }
        }

        Ok(Self {
            builder,
            mode,
            is_convert,
        })
    }

    #[napi(setter, js_name = "mode")]
    pub fn set_mode(&mut self, mode: GameMode) {
        self.mode = Some(mode);
        self.builder.mode(mode.into(), self.is_convert);
    }

    #[napi(setter, js_name = "isConvert")]
    pub fn set_is_convert(&mut self, is_convert: bool) {
        self.is_convert = is_convert;
        if let Some(m) = self.mode {
            self.builder.mode(m.into(), is_convert);
        }
    }

    #[napi(setter, js_name = "mods")]
    pub fn set_mods(&mut self, env: Env, mods: Option<Unknown>) -> Result<()> {
        if let Some(m) = mods {
            let info = parse_mods_full(&env, m)?;
            let game_mods = GameMods::from(info.mods);
            self.builder.mods(game_mods);
            if let Some(cr) = info.clock_rate {
                self.builder.clock_rate(cr);
            }
            if let Some(ar) = info.ar {
                self.builder.ar(ar as f32, false);
            }
            if let Some(cs) = info.cs {
                self.builder.cs(cs as f32, false);
            }
            if let Some(hp) = info.hp {
                self.builder.hp(hp as f32, false);
            }
            if let Some(od) = info.od {
                self.builder.od(od as f32, false);
            }
        }
        Ok(())
    }

    #[napi(setter, js_name = "clockRate")]
    pub fn set_clock_rate(&mut self, clock_rate: f64) {
        self.builder.clock_rate(clock_rate);
    }

    #[napi(setter, js_name = "ar")]
    pub fn set_ar(&mut self, ar: f64) {
        self.builder.ar(ar as f32, false);
    }

    #[napi(setter, js_name = "cs")]
    pub fn set_cs(&mut self, cs: f64) {
        self.builder.cs(cs as f32, false);
    }

    #[napi(setter, js_name = "hp")]
    pub fn set_hp(&mut self, hp: f64) {
        self.builder.hp(hp as f32, false);
    }

    #[napi(setter, js_name = "od")]
    pub fn set_od(&mut self, od: f64) {
        self.builder.od(od as f32, false);
    }

    #[napi(js_name = "build")]
    pub fn build(&self) -> JsBeatmapAttributes {
        let attrs = self.builder.build();
        let clock_rate = attrs.clock_rate();
        let raw_ar = attrs.ar() as f64;
        let raw_od = attrs.od() as f64;
        let raw_cs = attrs.cs() as f64;
        let raw_hp = attrs.hp() as f64;

        let mode = self.mode.unwrap_or(GameMode::Osu);
        let (mod_ar, mod_od) = match mode {
            GameMode::Osu | GameMode::Catch => (
                calculate_ar(raw_ar, clock_rate),
                calculate_od(raw_od, clock_rate),
            ),
            GameMode::Taiko => (
                raw_ar,
                calculate_taiko_od(raw_od, clock_rate),
            ),
            GameMode::Mania => (
                raw_ar,
                raw_od,
            ),
        };

        JsBeatmapAttributes {
            ar: mod_ar,
            cs: raw_cs,
            hp: raw_hp,
            od: mod_od,
            clock_rate,
        }
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}
