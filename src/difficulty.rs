use napi::bindgen_prelude::*;
use napi_derive::napi;
use rosu_pp::Difficulty;
use rosu_pp::any::DifficultyAttributes;
use crate::enums::GameMode;
use crate::mods::parse_mods_full;
use crate::beatmap::JsBeatmap;
use crate::strains::JsStrains;
use crate::gradual::{JsGradualDifficulty, JsGradualPerformance};
use crate::lazer_engine::*;
use crate::lazer_skills::*;
use crate::math::*;

#[napi(js_name = "DifficultyAttributes")]
#[derive(Debug, Clone)]
pub struct JsDifficultyAttributes {
    pub(crate) inner: DifficultyAttributes,
    pub(crate) ar_val: f64,
    pub(crate) cs_val: f64,
    pub(crate) hp_val: f64,
    pub(crate) od_val: f64,
    pub(crate) clock_rate_val: f64,
    pub(crate) aim_val: Option<f64>,
    pub(crate) speed_val: Option<f64>,
    pub(crate) reading_val: Option<f64>,
    pub(crate) flashlight_val: Option<f64>,
    pub(crate) slider_factor_val: Option<f64>,
    pub(crate) stars_val: Option<f64>,
    pub(crate) aim_difficult_strain_count_val: Option<f64>,
    pub(crate) speed_difficult_strain_count_val: Option<f64>,
    pub(crate) reading_difficult_note_count_val: Option<f64>,
    pub(crate) speed_note_count_val: Option<f64>,
    pub(crate) aim_difficult_slider_count_val: Option<f64>,
    pub(crate) aim_top_weighted_slider_factor_val: Option<f64>,
    pub(crate) speed_top_weighted_slider_factor_val: Option<f64>,
}

#[napi]
impl JsDifficultyAttributes {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Err(Error::new(Status::InvalidArg, "DifficultyAttributes cannot be constructed directly"))
    }

    pub fn from_rosu(inner: DifficultyAttributes, ar_val: f64, cs_val: f64, hp_val: f64, od_val: f64, clock_rate_val: f64) -> Self {
        Self {
            inner,
            ar_val,
            cs_val,
            hp_val,
            od_val,
            clock_rate_val,
            aim_val: None,
            speed_val: None,
            reading_val: None,
            flashlight_val: None,
            slider_factor_val: None,
            stars_val: None,
            aim_difficult_strain_count_val: None,
            speed_difficult_strain_count_val: None,
            reading_difficult_note_count_val: None,
            speed_note_count_val: None,
            aim_difficult_slider_count_val: None,
            aim_top_weighted_slider_factor_val: None,
            speed_top_weighted_slider_factor_val: None,
        }
    }

    #[napi(getter, js_name = "stars")]
    pub fn stars(&self) -> f64 {
        self.stars_val.unwrap_or_else(|| self.inner.stars())
    }

    #[napi(getter, js_name = "maxCombo")]
    pub fn max_combo(&self) -> u32 {
        self.inner.max_combo()
    }

    #[napi(getter, js_name = "mode")]
    pub fn mode(&self) -> GameMode {
        match &self.inner {
            DifficultyAttributes::Osu(_) => GameMode::Osu,
            DifficultyAttributes::Taiko(_) => GameMode::Taiko,
            DifficultyAttributes::Catch(_) => GameMode::Catch,
            DifficultyAttributes::Mania(_) => GameMode::Mania,
        }
    }

    #[napi(getter, js_name = "aim")]
    pub fn aim(&self) -> Option<f64> {
        self.aim_val.or_else(|| match &self.inner {
            DifficultyAttributes::Osu(a) => Some(a.aim),
            _ => None,
        })
    }

    #[napi(getter, js_name = "aimStars")]
    pub fn aim_stars(&self) -> Option<f64> {
        self.aim()
    }

    #[napi(getter, js_name = "speed")]
    pub fn speed(&self) -> Option<f64> {
        self.speed_val.or_else(|| match &self.inner {
            DifficultyAttributes::Osu(a) => Some(a.speed),
            _ => None,
        })
    }

    #[napi(getter, js_name = "speedStars")]
    pub fn speed_stars(&self) -> Option<f64> {
        self.speed()
    }

    #[napi(getter, js_name = "reading")]
    pub fn reading(&self) -> Option<f64> {
        self.reading_val
    }

    #[napi(getter, js_name = "readingStars")]
    pub fn reading_stars(&self) -> Option<f64> {
        self.reading_val
    }

    #[napi(getter, js_name = "flashlight")]
    pub fn flashlight(&self) -> Option<f64> {
        self.flashlight_val.or_else(|| match &self.inner {
            DifficultyAttributes::Osu(a) => Some(a.flashlight),
            _ => None,
        })
    }

    #[napi(getter, js_name = "flashlightStars")]
    pub fn flashlight_stars(&self) -> Option<f64> {
        self.flashlight()
    }

    #[napi(getter, js_name = "sliderFactor")]
    pub fn slider_factor(&self) -> Option<f64> {
        self.slider_factor_val.or_else(|| match &self.inner {
            DifficultyAttributes::Osu(a) => Some(a.slider_factor),
            _ => None,
        })
    }

    #[napi(getter, js_name = "speedNoteCount")]
    pub fn speed_note_count(&self) -> Option<f64> {
        self.speed_note_count_val.or_else(|| match &self.inner {
            DifficultyAttributes::Osu(a) => Some(a.speed_note_count),
            _ => None,
        })
    }

    #[napi(getter, js_name = "aimDifficultSliderCount")]
    pub fn aim_difficult_slider_count(&self) -> Option<f64> {
        self.aim_difficult_slider_count_val
    }

    #[napi(getter, js_name = "aimDifficultStrainCount")]
    pub fn aim_difficult_strain_count(&self) -> Option<f64> {
        self.aim_difficult_strain_count_val
    }

    #[napi(getter, js_name = "speedDifficultStrainCount")]
    pub fn speed_difficult_strain_count(&self) -> Option<f64> {
        self.speed_difficult_strain_count_val
    }

    #[napi(getter, js_name = "readingDifficultNoteCount")]
    pub fn reading_difficult_note_count(&self) -> Option<f64> {
        self.reading_difficult_note_count_val
    }

    #[napi(getter, js_name = "aimTopWeightedSliderFactor")]
    pub fn aim_top_weighted_slider_factor(&self) -> Option<f64> {
        self.aim_top_weighted_slider_factor_val
    }

    #[napi(getter, js_name = "speedTopWeightedSliderFactor")]
    pub fn speed_top_weighted_slider_factor(&self) -> Option<f64> {
        self.speed_top_weighted_slider_factor_val
    }

    #[napi(getter, js_name = "stamina")]
    pub fn stamina(&self) -> Option<f64> {
        match &self.inner {
            DifficultyAttributes::Taiko(a) => Some(a.stamina),
            _ => None,
        }
    }

    #[napi(getter, js_name = "rhythm")]
    pub fn rhythm(&self) -> Option<f64> {
        match &self.inner {
            DifficultyAttributes::Taiko(a) => Some(a.rhythm),
            _ => None,
        }
    }

    #[napi(getter, js_name = "color")]
    pub fn color(&self) -> Option<f64> {
        match &self.inner {
            DifficultyAttributes::Taiko(a) => Some(a.color),
            _ => None,
        }
    }

    #[napi(getter, js_name = "ar")]
    pub fn ar(&self) -> f64 {
        self.ar_val
    }

    #[napi(getter, js_name = "cs")]
    pub fn cs(&self) -> f64 {
        self.cs_val
    }

    #[napi(getter, js_name = "hp")]
    pub fn hp(&self) -> f64 {
        self.hp_val
    }

    #[napi(getter, js_name = "od")]
    pub fn od(&self) -> f64 {
        self.od_val
    }

    #[napi(getter, js_name = "clockRate")]
    pub fn clock_rate(&self) -> f64 {
        self.clock_rate_val
    }
}

#[napi(object)]
pub struct DifficultyArgs {
    pub mods: Option<Unknown>,
    pub clock_rate: Option<f64>,
    pub ar: Option<f64>,
    pub cs: Option<f64>,
    pub hp: Option<f64>,
    pub od: Option<f64>,
    pub passed_objects: Option<u32>,
    pub hardrock_offsets: Option<bool>,
    pub lazer: Option<bool>,
}

#[napi(js_name = "Difficulty")]
#[derive(Clone)]
pub struct JsDifficulty {
    pub(crate) difficulty: Difficulty,
    pub(crate) has_hidden: bool,
    pub(crate) has_hardrock: bool,
    pub(crate) has_easy: bool,
    pub(crate) has_flashlight: bool,
    pub(crate) custom_clock_rate: f64,
    pub(crate) custom_ar: Option<f64>,
    pub(crate) custom_cs: Option<f64>,
    pub(crate) custom_hp: Option<f64>,
    pub(crate) custom_od: Option<f64>,
}

#[napi]
impl JsDifficulty {
    #[napi(constructor)]
    pub fn new(env: Env, args: Option<DifficultyArgs>) -> Result<Self> {
        let mut diff = Difficulty::new();
        let mut has_hidden = false;
        let mut has_hardrock = false;
        let mut has_easy = false;
        let mut has_flashlight = false;
        let mut custom_clock_rate = 1.0;
        let mut custom_ar = None;
        let mut custom_cs = None;
        let mut custom_hp = None;
        let mut custom_od = None;

        if let Some(a) = args {
            if let Some(mods) = a.mods {
                let info = parse_mods_full(&env, mods)?;
                let mut d = diff.mods(info.mods.bits());
                if let Some(rate) = info.clock_rate {
                    d = d.clock_rate(rate);
                    custom_clock_rate = rate;
                }
                if let Some(ar) = info.ar {
                    d = d.ar(ar as f32, false);
                    custom_ar = Some(ar);
                }
                if let Some(cs) = info.cs {
                    d = d.cs(cs as f32, false);
                    custom_cs = Some(cs);
                }
                if let Some(hp) = info.hp {
                    d = d.hp(hp as f32, false);
                    custom_hp = Some(hp);
                }
                if let Some(od) = info.od {
                    d = d.od(od as f32, false);
                    custom_od = Some(od);
                }
                has_hidden = info.has_hidden;
                has_hardrock = info.has_hardrock;
                has_easy = info.has_easy;
                has_flashlight = info.has_flashlight;
                diff = d;
            }
            if let Some(rate) = a.clock_rate {
                diff = diff.clock_rate(rate);
                custom_clock_rate = rate;
            }
            if let Some(ar) = a.ar {
                diff = diff.ar(ar as f32, false);
                custom_ar = Some(ar);
            }
            if let Some(cs) = a.cs {
                diff = diff.cs(cs as f32, false);
                custom_cs = Some(cs);
            }
            if let Some(hp) = a.hp {
                diff = diff.hp(hp as f32, false);
                custom_hp = Some(hp);
            }
            if let Some(od) = a.od {
                diff = diff.od(od as f32, false);
                custom_od = Some(od);
            }
            if let Some(passed) = a.passed_objects {
                diff = diff.passed_objects(passed);
            }
            if let Some(hr_offsets) = a.hardrock_offsets {
                diff = diff.hardrock_offsets(hr_offsets);
            }
            if let Some(lazer) = a.lazer {
                diff = diff.lazer(lazer);
            }
        }

        Ok(Self {
            difficulty: diff,
            has_hidden,
            has_hardrock,
            has_easy,
            has_flashlight,
            custom_clock_rate,
            custom_ar,
            custom_cs,
            custom_hp,
            custom_od,
        })
    }

    #[napi(setter, js_name = "mods")]
    pub fn set_mods(&mut self, env: Env, mods: Unknown) -> Result<()> {
        let info = parse_mods_full(&env, mods)?;
        let mut d = self.difficulty.clone().mods(info.mods.bits());
        if let Some(rate) = info.clock_rate {
            d = d.clock_rate(rate);
            self.custom_clock_rate = rate;
        }
        if let Some(ar) = info.ar {
            d = d.ar(ar as f32, false);
            self.custom_ar = Some(ar);
        }
        if let Some(cs) = info.cs {
            d = d.cs(cs as f32, false);
            self.custom_cs = Some(cs);
        }
        if let Some(hp) = info.hp {
            d = d.hp(hp as f32, false);
            self.custom_hp = Some(hp);
        }
        if let Some(od) = info.od {
            d = d.od(od as f32, false);
            self.custom_od = Some(od);
        }
        self.has_hidden = info.has_hidden;
        self.has_hardrock = info.has_hardrock;
        self.has_easy = info.has_easy;
        self.has_flashlight = info.has_flashlight;
        self.difficulty = d;
        Ok(())
    }

    #[napi(setter, js_name = "clockRate")]
    pub fn set_clock_rate(&mut self, clock_rate: f64) {
        self.custom_clock_rate = clock_rate;
        self.difficulty = self.difficulty.clone().clock_rate(clock_rate);
    }

    #[napi(setter, js_name = "ar")]
    pub fn set_ar(&mut self, ar: f64) {
        self.custom_ar = Some(ar);
        self.difficulty = self.difficulty.clone().ar(ar as f32, false);
    }

    #[napi(setter, js_name = "cs")]
    pub fn set_cs(&mut self, cs: f64) {
        self.custom_cs = Some(cs);
        self.difficulty = self.difficulty.clone().cs(cs as f32, false);
    }

    #[napi(setter, js_name = "hp")]
    pub fn set_hp(&mut self, hp: f64) {
        self.custom_hp = Some(hp);
        self.difficulty = self.difficulty.clone().hp(hp as f32, false);
    }

    #[napi(setter, js_name = "od")]
    pub fn set_od(&mut self, od: f64) {
        self.custom_od = Some(od);
        self.difficulty = self.difficulty.clone().od(od as f32, false);
    }

    #[napi(setter, js_name = "passedObjects")]
    pub fn set_passed_objects(&mut self, passed_objects: u32) {
        self.difficulty = self.difficulty.clone().passed_objects(passed_objects);
    }

    #[napi(setter, js_name = "hardrockOffsets")]
    pub fn set_hardrock_offsets(&mut self, hardrock_offsets: bool) {
        self.difficulty = self.difficulty.clone().hardrock_offsets(hardrock_offsets);
    }

    #[napi(setter, js_name = "lazer")]
    pub fn set_lazer(&mut self, lazer: bool) {
        self.difficulty = self.difficulty.clone().lazer(lazer);
    }

    #[napi(js_name = "calculate")]
    pub fn calculate(&self, map: &JsBeatmap) -> JsDifficultyAttributes {
        let attrs = self.difficulty.calculate(&map.inner);
        let mut ar = self.custom_ar.unwrap_or(map.inner.ar as f64);
        let mut cs = self.custom_cs.unwrap_or(map.inner.cs as f64);
        let mut hp = self.custom_hp.unwrap_or(map.inner.hp as f64);
        let mut od = self.custom_od.unwrap_or(map.inner.od as f64);
        let clock_rate = self.custom_clock_rate;

        if self.custom_cs.is_none() {
            if self.has_hardrock {
                cs = (cs * 1.3).min(10.0);
            } else if self.has_easy {
                cs *= 0.5;
            }
        }
        if self.custom_ar.is_none() {
            if self.has_hardrock {
                ar = (ar * 1.4).min(10.0);
            } else if self.has_easy {
                ar *= 0.5;
            }
        }
        if self.custom_od.is_none() {
            if self.has_hardrock {
                od = (od * 1.4).min(10.0);
            } else if self.has_easy {
                od *= 0.5;
            }
        }
        if self.custom_hp.is_none() {
            if self.has_hardrock {
                hp = (hp * 1.4).min(10.0);
            } else if self.has_easy {
                hp *= 0.5;
            }
        }

        let mut aim_val = None;
        let mut speed_val = None;
        let mut reading_val = None;
        let mut flashlight_val = None;
        let mut slider_factor_val = None;
        let mut stars_val = None;
        let mut aim_difficult_strain_count_val = None;
        let mut speed_difficult_strain_count_val = None;
        let mut reading_difficult_note_count_val = None;
        let mut speed_note_count_val = None;
        let mut aim_difficult_slider_count_val = None;
        let mut aim_top_weighted_slider_factor_val = None;
        let mut speed_top_weighted_slider_factor_val = None;

        if let DifficultyAttributes::Osu(_) = attrs {
            let lazer_objects = build_lazer_difficulty_hit_objects(&map.inner, ar, cs, od, clock_rate, self.has_hidden);
            if !lazer_objects.is_empty() {
                // 1. Aim Skill (with sliders)
                let mut aim_skill_sliders = LazerAimSkill::new(true);
                aim_skill_sliders.process_objects(&lazer_objects);
                let num_aim_diff = aim_skill_sliders.difficulty_value();
                let aim_rating = num_aim_diff.powf(0.63) * 0.02275;
                let aim_difficult_strain_count = aim_skill_sliders.count_top_weighted_strains(num_aim_diff);
                let difficult_sliders = aim_skill_sliders.get_difficult_sliders();

                // 2. Aim Skill (without sliders)
                let mut aim_skill_no_sliders = LazerAimSkill::new(false);
                aim_skill_no_sliders.process_objects(&lazer_objects);
                let num_aim_no_sliders_diff = aim_skill_no_sliders.difficulty_value();
                let aim_no_sliders = num_aim_no_sliders_diff.powf(0.63) * 0.02275;
                let num3 = aim_skill_no_sliders.count_top_weighted_sliders(num_aim_no_sliders_diff);
                let num4 = aim_skill_no_sliders.count_top_weighted_strains(num_aim_no_sliders_diff);
                let aim_top_weighted_slider_factor = num3 / (num4 - num3).max(1.0);

                let slider_factor = if num_aim_diff > 0.0 {
                    (aim_no_sliders / aim_rating).clamp(0.0, 1.0)
                } else {
                    1.0
                };

                // 3. Speed Skill
                let mut speed_skill = LazerSpeedSkill::new();
                speed_skill.process_objects(&lazer_objects);
                let speed_diff_val = speed_skill.difficulty_value();
                let speed_rating = speed_diff_val.sqrt() * 0.0675;
                let speed_difficult_strain_count = speed_skill.count_top_weighted_object_difficulties(speed_diff_val);
                let speed_note_count = speed_skill.relevant_object_count();
                let num5 = speed_skill.count_top_weighted_sliders(speed_diff_val);
                let speed_top_weighted_slider_factor = num5 / (speed_difficult_strain_count - num5).max(1.0);

                // 4. Reading Skill
                let mut reading_skill = LazerReadingSkill::new(self.has_hidden);
                reading_skill.process_objects(&lazer_objects);
                let reading_diff_val = reading_skill.difficulty_value();
                let reading_rating = reading_diff_val.sqrt() * 0.0675;
                let reading_difficult_note_count = reading_skill.count_top_weighted_object_difficulties(reading_diff_val);

                // 5. Flashlight Skill
                let mut fl_rating = 0.0;
                if self.has_flashlight {
                    let mut fl_skill = LazerFlashlightSkill::new(lazer_objects.len() + 1, self.has_hidden);
                    fl_skill.process_objects(&lazer_objects);
                    fl_rating = fl_skill.calculate_rating();
                }

                aim_val = Some(aim_rating);
                speed_val = Some(speed_rating);
                reading_val = Some(reading_rating);
                flashlight_val = Some(fl_rating);
                slider_factor_val = Some(slider_factor);
                aim_difficult_strain_count_val = Some(aim_difficult_strain_count);
                speed_difficult_strain_count_val = Some(speed_difficult_strain_count);
                reading_difficult_note_count_val = Some(reading_difficult_note_count);
                speed_note_count_val = Some(speed_note_count);
                aim_difficult_slider_count_val = Some(difficult_sliders);
                aim_top_weighted_slider_factor_val = Some(aim_top_weighted_slider_factor);
                speed_top_weighted_slider_factor_val = Some(speed_top_weighted_slider_factor);

                // Star Rating
                let aim_perf = 4.0 * aim_rating.powi(3);
                let speed_perf = 4.0 * speed_rating.powi(3);
                let reading_perf = 4.0 * reading_rating.powi(3);
                let fl_perf = 25.0 * fl_rating.powi(2);

                let cognition = sum_cognition_difficulty(reading_perf, fl_perf);
                let base_perf = norm(1.1, &[aim_perf, speed_perf, cognition]);
                let star_rating = calculate_star_rating(base_perf);
                stars_val = Some(star_rating);
            }
        }

        JsDifficultyAttributes {
            inner: attrs,
            ar_val: ar,
            cs_val: cs,
            hp_val: hp,
            od_val: od,
            clock_rate_val: clock_rate,
            aim_val,
            speed_val,
            reading_val,
            flashlight_val,
            slider_factor_val,
            stars_val,
            aim_difficult_strain_count_val,
            speed_difficult_strain_count_val,
            reading_difficult_note_count_val,
            speed_note_count_val,
            aim_difficult_slider_count_val,
            aim_top_weighted_slider_factor_val,
            speed_top_weighted_slider_factor_val,
        }
    }

    #[napi(js_name = "strains")]
    pub fn strains(&self, map: &JsBeatmap) -> JsStrains {
        let s = self.difficulty.strains(&map.inner);
        JsStrains::from_rosu(s)
    }

    #[napi(js_name = "gradualDifficulty")]
    pub fn gradual_difficulty(&self, map: &JsBeatmap) -> JsGradualDifficulty {
        let gd = self.difficulty.clone().gradual_difficulty(&map.inner);
        JsGradualDifficulty::from_rosu(gd, map.inner.ar as f64, map.inner.cs as f64, map.inner.hp as f64, map.inner.od as f64)
    }

    #[napi(js_name = "gradualPerformance")]
    pub fn gradual_performance(&self, map: &JsBeatmap) -> JsGradualPerformance {
        let gp = self.difficulty.clone().gradual_performance(&map.inner);
        JsGradualPerformance::from_rosu(gp, map.inner.ar as f64, map.inner.cs as f64, map.inner.hp as f64, map.inner.od as f64)
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}
