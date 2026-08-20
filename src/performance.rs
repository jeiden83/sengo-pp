use napi::bindgen_prelude::*;
use napi_derive::napi;
use napi::Env;
use rosu_pp::{Difficulty, GameMods, Performance};
use rosu_pp::any::{PerformanceAttributes, ScoreState as RosuScoreState};
use crate::enums::HitResultPriority;
use crate::mods::parse_mods_full;
use crate::beatmap::JsBeatmap;
use crate::difficulty::JsDifficultyAttributes;
use crate::reading::{ReadingEvaluator, ReadingHitObject, ReadingSkill};

#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct ScoreState {
    pub max_combo: Option<u32>,
    pub osu_large_tick_hits: Option<u32>,
    pub osu_small_tick_hits: Option<u32>,
    pub slider_end_hits: Option<u32>,
    pub n_geki: Option<u32>,
    pub n_katu: Option<u32>,
    pub n300: Option<u32>,
    pub n100: Option<u32>,
    pub n50: Option<u32>,
    pub misses: Option<u32>,
    pub legacy_total_score: Option<u32>,
}

impl From<RosuScoreState> for ScoreState {
    fn from(s: RosuScoreState) -> Self {
        Self {
            max_combo: Some(s.max_combo),
            osu_large_tick_hits: Some(s.osu_large_tick_hits),
            osu_small_tick_hits: Some(s.osu_small_tick_hits),
            slider_end_hits: Some(s.slider_end_hits),
            n_geki: Some(s.n_geki),
            n_katu: Some(s.n_katu),
            n300: Some(s.n300),
            n100: Some(s.n100),
            n50: Some(s.n50),
            misses: Some(s.misses),
            legacy_total_score: s.legacy_total_score,
        }
    }
}

impl From<ScoreState> for RosuScoreState {
    fn from(s: ScoreState) -> Self {
        Self {
            max_combo: s.max_combo.unwrap_or(0),
            osu_large_tick_hits: s.osu_large_tick_hits.unwrap_or(0),
            osu_small_tick_hits: s.osu_small_tick_hits.unwrap_or(0),
            slider_end_hits: s.slider_end_hits.unwrap_or(0),
            n_geki: s.n_geki.unwrap_or(0),
            n_katu: s.n_katu.unwrap_or(0),
            n300: s.n300.unwrap_or(0),
            n100: s.n100.unwrap_or(0),
            n50: s.n50.unwrap_or(0),
            misses: s.misses.unwrap_or(0),
            legacy_total_score: s.legacy_total_score,
        }
    }
}

#[napi(js_name = "PerformanceAttributes")]
#[derive(Debug, Clone)]
pub struct JsPerformanceAttributes {
    pub(crate) inner: PerformanceAttributes,
    pub(crate) state_val: Option<ScoreState>,
    pub(crate) ar_val: f64,
    pub(crate) cs_val: f64,
    pub(crate) hp_val: f64,
    pub(crate) od_val: f64,
    pub(crate) clock_rate_val: f64,
    pub(crate) pp_aim_val: Option<f64>,
    pub(crate) pp_speed_val: Option<f64>,
    pub(crate) pp_acc_val: Option<f64>,
    pub(crate) pp_reading_val: Option<f64>,
    pub(crate) pp_fl_val: Option<f64>,
    pub(crate) pp_total_val: Option<f64>,
}

#[napi]
impl JsPerformanceAttributes {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Err(Error::new(Status::InvalidArg, "PerformanceAttributes cannot be constructed directly"))
    }

    #[napi(getter, js_name = "pp")]
    pub fn pp(&self) -> f64 {
        if let Some(p) = self.pp_total_val {
            p
        } else {
            self.inner.pp()
        }
    }

    #[napi(getter, js_name = "ppAim")]
    pub fn pp_aim(&self) -> Option<f64> {
        self.pp_aim_val.or_else(|| match &self.inner {
            PerformanceAttributes::Osu(a) => Some(a.pp_aim),
            _ => None,
        })
    }

    #[napi(getter, js_name = "ppFlashlight")]
    pub fn pp_flashlight(&self) -> Option<f64> {
        self.pp_fl_val.or_else(|| match &self.inner {
            PerformanceAttributes::Osu(a) => Some(a.pp_flashlight),
            _ => None,
        })
    }

    #[napi(getter, js_name = "ppReading")]
    pub fn pp_reading(&self) -> Option<f64> {
        self.pp_reading_val
    }

    #[napi(getter, js_name = "ppSpeed")]
    pub fn pp_speed(&self) -> Option<f64> {
        self.pp_speed_val.or_else(|| match &self.inner {
            PerformanceAttributes::Osu(a) => Some(a.pp_speed),
            _ => None,
        })
    }

    #[napi(getter, js_name = "ppAccuracy")]
    pub fn pp_accuracy(&self) -> Option<f64> {
        self.pp_acc_val.or_else(|| match &self.inner {
            PerformanceAttributes::Osu(a) => Some(a.pp_acc),
            PerformanceAttributes::Taiko(a) => Some(a.pp_acc),
            _ => None,
        })
    }

    #[napi(getter, js_name = "ppAcc")]
    pub fn pp_acc(&self) -> Option<f64> {
        self.pp_accuracy()
    }

    #[napi(getter, js_name = "ppDifficulty")]
    pub fn pp_difficulty(&self) -> Option<f64> {
        match &self.inner {
            PerformanceAttributes::Taiko(a) => Some(a.pp_difficulty),
            PerformanceAttributes::Mania(a) => Some(a.pp_difficulty),
            _ => None,
        }
    }

    #[napi(getter, js_name = "effectiveMissCount")]
    pub fn effective_miss_count(&self) -> Option<f64> {
        match &self.inner {
            PerformanceAttributes::Osu(a) => Some(a.effective_miss_count),
            _ => None,
        }
    }

    #[napi(getter, js_name = "speedDeviation")]
    pub fn speed_deviation(&self) -> Option<f64> {
        match &self.inner {
            PerformanceAttributes::Osu(a) => a.speed_deviation,
            _ => None,
        }
    }

    #[napi(getter, js_name = "estimatedUnstableRate")]
    pub fn estimated_unstable_rate(&self) -> Option<f64> {
        match &self.inner {
            PerformanceAttributes::Taiko(a) => a.estimated_unstable_rate,
            _ => None,
        }
    }

    #[napi(getter, js_name = "difficulty")]
    pub fn difficulty(&self) -> JsDifficultyAttributes {
        JsDifficultyAttributes::from_rosu(
            self.inner.difficulty_attributes(),
            self.ar_val,
            self.cs_val,
            self.hp_val,
            self.od_val,
            self.clock_rate_val,
        )
    }

    #[napi(getter, js_name = "state")]
    pub fn state(&self) -> Option<ScoreState> {
        self.state_val.clone()
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}

#[napi(object)]
pub struct PerformanceArgs {
    pub mods: Option<Unknown>,
    pub accuracy: Option<f64>,
    pub combo: Option<u32>,
    pub misses: Option<u32>,
    pub n300: Option<u32>,
    pub n100: Option<u32>,
    pub n50: Option<u32>,
    pub n_geki: Option<u32>,
    pub n_katu: Option<u32>,
    pub slider_end_hits: Option<u32>,
    pub large_tick_hits: Option<u32>,
    pub small_tick_hits: Option<u32>,
    pub clock_rate: Option<f64>,
    pub passed_objects: Option<u32>,
    pub ar: Option<f64>,
    pub cs: Option<f64>,
    pub hp: Option<f64>,
    pub od: Option<f64>,
    pub lazer: Option<bool>,
    pub legacy_total_score: Option<u32>,
    pub hitresult_priority: Option<HitResultPriority>,
}

#[napi(js_name = "Performance")]
pub struct JsPerformance {
    pub(crate) mods: Option<GameMods>,
    pub(crate) accuracy: Option<f64>,
    pub(crate) combo: Option<u32>,
    pub(crate) misses: Option<u32>,
    pub(crate) n300: Option<u32>,
    pub(crate) n100: Option<u32>,
    pub(crate) n50: Option<u32>,
    pub(crate) n_geki: Option<u32>,
    pub(crate) n_katu: Option<u32>,
    pub(crate) slider_end_hits: Option<u32>,
    pub(crate) large_tick_hits: Option<u32>,
    pub(crate) small_tick_hits: Option<u32>,
    pub(crate) clock_rate: Option<f64>,
    pub(crate) passed_objects: Option<u32>,
    pub(crate) ar: Option<f64>,
    pub(crate) cs: Option<f64>,
    pub(crate) hp: Option<f64>,
    pub(crate) od: Option<f64>,
    pub(crate) lazer: Option<bool>,
    pub(crate) legacy_total_score: Option<u32>,
    pub(crate) hitresult_priority: Option<HitResultPriority>,
    pub(crate) has_hidden: bool,
    pub(crate) has_flashlight: bool,
}

#[napi]
impl JsPerformance {
    #[napi(constructor)]
    pub fn new(env: Env, args: Option<PerformanceArgs>) -> Result<Self> {
        let mut inst = Self {
            mods: None,
            accuracy: None,
            combo: None,
            misses: None,
            n300: None,
            n100: None,
            n50: None,
            n_geki: None,
            n_katu: None,
            slider_end_hits: None,
            large_tick_hits: None,
            small_tick_hits: None,
            clock_rate: None,
            passed_objects: None,
            ar: None,
            cs: None,
            hp: None,
            od: None,
            lazer: None,
            legacy_total_score: None,
            hitresult_priority: None,
            has_hidden: false,
            has_flashlight: false,
        };

        if let Some(a) = args {
            if let Some(m) = a.mods {
                let info = parse_mods_full(&env, m)?;
                inst.mods = Some(GameMods::from(info.mods.clone()));
                if let Some(cr) = info.clock_rate {
                    inst.clock_rate = Some(cr);
                }
                if let Some(ar) = info.ar {
                    inst.ar = Some(ar);
                }
                if let Some(cs) = info.cs {
                    inst.cs = Some(cs);
                }
                if let Some(hp) = info.hp {
                    inst.hp = Some(hp);
                }
                if let Some(od) = info.od {
                    inst.od = Some(od);
                }
                if info.mods.bits() & 8 != 0 {
                    inst.has_hidden = true;
                }
                if info.mods.bits() & 1024 != 0 {
                    inst.has_flashlight = true;
                }
            }
            inst.accuracy = a.accuracy;
            inst.combo = a.combo;
            inst.misses = a.misses;
            inst.n300 = a.n300;
            inst.n100 = a.n100;
            inst.n50 = a.n50;
            inst.n_geki = a.n_geki;
            inst.n_katu = a.n_katu;
            inst.slider_end_hits = a.slider_end_hits;
            inst.large_tick_hits = a.large_tick_hits;
            inst.small_tick_hits = a.small_tick_hits;
            if a.clock_rate.is_some() {
                inst.clock_rate = a.clock_rate;
            }
            inst.passed_objects = a.passed_objects;
            if a.ar.is_some() {
                inst.ar = a.ar;
            }
            if a.cs.is_some() {
                inst.cs = a.cs;
            }
            if a.hp.is_some() {
                inst.hp = a.hp;
            }
            if a.od.is_some() {
                inst.od = a.od;
            }
            inst.lazer = a.lazer;
            inst.legacy_total_score = a.legacy_total_score;
            inst.hitresult_priority = a.hitresult_priority;
        }

        Ok(inst)
    }

    #[napi(setter, js_name = "mods")]
    pub fn set_mods(&mut self, env: Env, mods: Unknown) -> Result<()> {
        let info = parse_mods_full(&env, mods)?;
        self.mods = Some(GameMods::from(info.mods.clone()));
        if let Some(cr) = info.clock_rate {
            self.clock_rate = Some(cr);
        }
        if let Some(ar) = info.ar {
            self.ar = Some(ar);
        }
        if let Some(cs) = info.cs {
            self.cs = Some(cs);
        }
        if let Some(hp) = info.hp {
            self.hp = Some(hp);
        }
        if let Some(od) = info.od {
            self.od = Some(od);
        }
        if info.mods.bits() & 8 != 0 {
            self.has_hidden = true;
        }
        if info.mods.bits() & 1024 != 0 {
            self.has_flashlight = true;
        }
        Ok(())
    }

    #[napi(setter, js_name = "accuracy")]
    pub fn set_accuracy(&mut self, accuracy: Option<f64>) {
        self.accuracy = accuracy;
    }

    #[napi(setter, js_name = "combo")]
    pub fn set_combo(&mut self, combo: Option<u32>) {
        self.combo = combo;
    }

    #[napi(setter, js_name = "misses")]
    pub fn set_misses(&mut self, misses: Option<u32>) {
        self.misses = misses;
    }

    #[napi(setter, js_name = "n300")]
    pub fn set_n300(&mut self, n300: Option<u32>) {
        self.n300 = n300;
    }

    #[napi(setter, js_name = "n100")]
    pub fn set_n100(&mut self, n100: Option<u32>) {
        self.n100 = n100;
    }

    #[napi(setter, js_name = "n50")]
    pub fn set_n50(&mut self, n50: Option<u32>) {
        self.n50 = n50;
    }

    #[napi(setter, js_name = "nGeki")]
    pub fn set_n_geki(&mut self, n_geki: Option<u32>) {
        self.n_geki = n_geki;
    }

    #[napi(setter, js_name = "nKatu")]
    pub fn set_n_katu(&mut self, n_katu: Option<u32>) {
        self.n_katu = n_katu;
    }

    #[napi(setter, js_name = "sliderEndHits")]
    pub fn set_slider_end_hits(&mut self, slider_end_hits: Option<u32>) {
        self.slider_end_hits = slider_end_hits;
    }

    #[napi(setter, js_name = "largeTickHits")]
    pub fn set_large_tick_hits(&mut self, large_tick_hits: Option<u32>) {
        self.large_tick_hits = large_tick_hits;
    }

    #[napi(setter, js_name = "smallTickHits")]
    pub fn set_small_tick_hits(&mut self, small_tick_hits: Option<u32>) {
        self.small_tick_hits = small_tick_hits;
    }

    #[napi(setter, js_name = "clockRate")]
    pub fn set_clock_rate(&mut self, clock_rate: Option<f64>) {
        self.clock_rate = clock_rate;
    }

    #[napi(setter, js_name = "passedObjects")]
    pub fn set_passed_objects(&mut self, passed_objects: Option<u32>) {
        self.passed_objects = passed_objects;
    }

    #[napi(setter, js_name = "ar")]
    pub fn set_ar(&mut self, ar: Option<f64>) {
        self.ar = ar;
    }

    #[napi(setter, js_name = "cs")]
    pub fn set_cs(&mut self, cs: Option<f64>) {
        self.cs = cs;
    }

    #[napi(setter, js_name = "hp")]
    pub fn set_hp(&mut self, hp: Option<f64>) {
        self.hp = hp;
    }

    #[napi(setter, js_name = "od")]
    pub fn set_od(&mut self, od: Option<f64>) {
        self.od = od;
    }

    #[napi(setter, js_name = "lazer")]
    pub fn set_lazer(&mut self, lazer: Option<bool>) {
        self.lazer = lazer;
    }

    fn apply_params<'a>(&self, mut perf: Performance<'a>) -> Performance<'a> {
        if let Some(ref m) = self.mods {
            perf = perf.mods(m.clone());
        }
        if let Some(acc) = self.accuracy {
            perf = perf.accuracy(acc);
        }
        if let Some(c) = self.combo {
            perf = perf.combo(c);
        }
        if let Some(m) = self.misses {
            perf = perf.misses(m);
        }
        if let Some(n) = self.n300 {
            perf = perf.n300(n);
        }
        if let Some(n) = self.n100 {
            perf = perf.n100(n);
        }
        if let Some(n) = self.n50 {
            perf = perf.n50(n);
        }
        if let Some(n) = self.n_geki {
            perf = perf.n_geki(n);
        }
        if let Some(n) = self.n_katu {
            perf = perf.n_katu(n);
        }
        if let Some(n) = self.slider_end_hits {
            perf = perf.slider_end_hits(n);
        }
        if let Some(n) = self.large_tick_hits {
            perf = perf.large_tick_hits(n);
        }
        if let Some(n) = self.small_tick_hits {
            perf = perf.small_tick_hits(n);
        }
        if let Some(cr) = self.clock_rate {
            perf = perf.clock_rate(cr);
        }
        if let Some(po) = self.passed_objects {
            perf = perf.passed_objects(po);
        }
        if let Some(ar) = self.ar {
            perf = perf.ar(ar as f32, false);
        }
        if let Some(cs) = self.cs {
            perf = perf.cs(cs as f32, false);
        }
        if let Some(hp) = self.hp {
            perf = perf.hp(hp as f32, false);
        }
        if let Some(od) = self.od {
            perf = perf.od(od as f32, false);
        }
        if let Some(lz) = self.lazer {
            perf = perf.lazer(lz);
        }
        perf
    }

    #[napi(js_name = "calculateBeatmap")]
    pub fn calculate_beatmap(&self, map: &JsBeatmap) -> JsPerformanceAttributes {
        let mut diff = Difficulty::new();
        if let Some(ref m) = self.mods {
            diff = diff.mods(m.clone());
        }
        if let Some(cr) = self.clock_rate {
            diff = diff.clock_rate(cr);
        }
        if let Some(ar) = self.ar {
            diff = diff.ar(ar as f32, false);
        }
        if let Some(cs) = self.cs {
            diff = diff.cs(cs as f32, false);
        }
        if let Some(hp) = self.hp {
            diff = diff.hp(hp as f32, false);
        }
        if let Some(od) = self.od {
            diff = diff.od(od as f32, false);
        }
        if let Some(po) = self.passed_objects {
            diff = diff.passed_objects(po);
        }
        if let Some(lz) = self.lazer {
            diff = diff.lazer(lz);
        }
        let diff_attrs = diff.calculate(&map.inner);
        let ar = self.ar.unwrap_or(map.inner.ar as f64);
        let cs = self.cs.unwrap_or(map.inner.cs as f64);
        let hp = self.hp.unwrap_or(map.inner.hp as f64);
        let od = self.od.unwrap_or(map.inner.od as f64);
        let cr = self.clock_rate.unwrap_or(1.0);

        let perf = Performance::new(diff_attrs);
        let perf = self.apply_params(perf);
        let perf_attrs = perf.calculate();

        let diff_obj = crate::difficulty::JsDifficulty::new(
            // Use existing difficulty builder
            unsafe { napi::Env::from_raw(std::ptr::null_mut()) },
            None,
        ).unwrap_or_else(|_| crate::difficulty::JsDifficulty {
            difficulty: diff.clone(),
            has_hidden: self.has_hidden,
            has_hardrock: false,
            has_easy: false,
            has_flashlight: self.has_flashlight,
            custom_clock_rate: cr,
            custom_ar: self.ar,
            custom_cs: self.cs,
            custom_hp: self.hp,
            custom_od: self.od,
        });

        let js_diff_attrs = diff_obj.calculate(map);
        self.calculate_attributes(&js_diff_attrs)
    }

    #[napi(js_name = "calculateAttributes")]
    pub fn calculate_attributes(&self, attrs: &JsDifficultyAttributes) -> JsPerformanceAttributes {
        let ar = self.ar.unwrap_or(attrs.ar_val);
        let cs = self.cs.unwrap_or(attrs.cs_val);
        let hp = self.hp.unwrap_or(attrs.hp_val);
        let od = self.od.unwrap_or(attrs.od_val);
        let cr = self.clock_rate.unwrap_or(attrs.clock_rate_val);

        let perf = Performance::new(attrs.inner.clone());
        let perf = self.apply_params(perf);
        let perf_attrs = perf.calculate();

        let mut pp_aim_val = None;
        let mut pp_speed_val = None;
        let mut pp_acc_val = None;
        let mut pp_reading_val = None;
        let mut pp_fl_val = None;
        let mut pp_total_val = None;

        if let PerformanceAttributes::Osu(_) = perf_attrs {
            let (hit_circles, sliders, spinners) = match &attrs.inner {
                rosu_pp::any::DifficultyAttributes::Osu(a) => (a.n_circles, a.n_sliders, a.n_spinners),
                _ => (0, 0, 0),
            };
            let max_combo = attrs.max_combo();

            let diff_result = crate::lazer_skills::LazerDifficultyResult {
                star_rating: attrs.stars(),
                aim_difficulty: attrs.aim().unwrap_or(0.0),
                speed_difficulty: attrs.speed().unwrap_or(0.0),
                reading_difficulty: attrs.reading_val.unwrap_or(0.0),
                flashlight_difficulty: attrs.flashlight().unwrap_or(0.0),
                slider_factor: attrs.slider_factor().unwrap_or(1.0),
                aim_difficult_strain_count: attrs.aim_difficult_strain_count_val.unwrap_or(1.0),
                speed_difficult_strain_count: attrs.speed_difficult_strain_count_val.unwrap_or(1.0),
                reading_difficult_note_count: attrs.reading_difficult_note_count_val.unwrap_or(1.0),
                speed_note_count: attrs.speed_note_count_val.unwrap_or(0.0),
                aim_top_weighted_slider_factor: attrs.aim_top_weighted_slider_factor_val.unwrap_or(0.0),
                speed_top_weighted_slider_factor: attrs.speed_top_weighted_slider_factor_val.unwrap_or(0.0),
                aim_difficult_slider_count: attrs.aim_difficult_slider_count_val.unwrap_or(0.0),
                max_combo,
                hit_circle_count: hit_circles,
                slider_count: sliders,
                spinner_count: spinners,
            };

            let acc = self.accuracy.unwrap_or(100.0);
            let misses = self.misses.unwrap_or(0);
            let has_flashlight = self.has_flashlight;

            let res = crate::lazer_skills::calculate_lazer_performance(
                &diff_result,
                acc,
                self.combo,
                misses,
                od,
                ar,
                cr,
                self.has_hidden,
                has_flashlight,
            );

            pp_aim_val = Some(res.aim_pp);
            pp_speed_val = Some(res.speed_pp);
            pp_acc_val = Some(res.accuracy_pp);
            pp_reading_val = Some(res.reading_pp);
            pp_fl_val = Some(res.flashlight_pp);
            pp_total_val = Some(res.pp);
        }

        JsPerformanceAttributes {
            inner: perf_attrs,
            state_val: None,
            ar_val: ar,
            cs_val: cs,
            hp_val: hp,
            od_val: od,
            clock_rate_val: cr,
            pp_aim_val,
            pp_speed_val,
            pp_acc_val,
            pp_reading_val,
            pp_fl_val,
            pp_total_val,
        }
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}

