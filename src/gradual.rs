use napi::bindgen_prelude::*;
use napi_derive::napi;
use rosu_pp::any::{GradualDifficulty, GradualPerformance, ScoreState as RosuScoreState};
use crate::difficulty::JsDifficultyAttributes;
use crate::performance::{JsPerformanceAttributes, ScoreState};

#[napi(js_name = "GradualDifficulty")]
pub struct JsGradualDifficulty {
    pub(crate) inner: Option<GradualDifficulty>,
    pub(crate) ar_val: f64,
    pub(crate) cs_val: f64,
    pub(crate) hp_val: f64,
    pub(crate) od_val: f64,
}

impl JsGradualDifficulty {
    pub fn from_rosu(inner: GradualDifficulty, ar: f64, cs: f64, hp: f64, od: f64) -> Self {
        Self {
            inner: Some(inner),
            ar_val: ar,
            cs_val: cs,
            hp_val: hp,
            od_val: od,
        }
    }
}

#[napi]
impl JsGradualDifficulty {
    #[napi(constructor)]
    pub fn dummy_constructor() -> Result<Self> {
        Err(Error::new(Status::InvalidArg, "GradualDifficulty cannot be constructed directly"))
    }

    #[napi(js_name = "nth")]
    pub fn nth(&mut self, n: u32) -> Option<JsDifficultyAttributes> {
        let ar = self.ar_val;
        let cs = self.cs_val;
        let hp = self.hp_val;
        let od = self.od_val;
        self.inner.as_mut()?.nth(n as usize).map(|attrs| JsDifficultyAttributes::from_rosu(attrs, ar, cs, hp, od, 1.0))
    }

    #[napi(js_name = "next")]
    pub fn next(&mut self) -> Option<JsDifficultyAttributes> {
        let ar = self.ar_val;
        let cs = self.cs_val;
        let hp = self.hp_val;
        let od = self.od_val;
        self.inner.as_mut()?.next().map(|attrs| JsDifficultyAttributes::from_rosu(attrs, ar, cs, hp, od, 1.0))
    }

    #[napi(js_name = "collect")]
    pub fn collect(&mut self) -> Vec<JsDifficultyAttributes> {
        let mut results = Vec::new();
        let ar = self.ar_val;
        let cs = self.cs_val;
        let hp = self.hp_val;
        let od = self.od_val;
        if let Some(ref mut inner) = self.inner {
            while let Some(attrs) = inner.next() {
                results.push(JsDifficultyAttributes::from_rosu(attrs, ar, cs, hp, od, 1.0));
            }
        }
        results
    }

    #[napi(getter, js_name = "nRemaining")]
    pub fn n_remaining(&self) -> u32 {
        self.inner.as_ref().map(|i| i.len() as u32).unwrap_or(0)
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}

#[napi(js_name = "GradualPerformance")]
pub struct JsGradualPerformance {
    pub(crate) inner: Option<GradualPerformance>,
    pub(crate) ar_val: f64,
    pub(crate) cs_val: f64,
    pub(crate) hp_val: f64,
    pub(crate) od_val: f64,
}

impl JsGradualPerformance {
    pub fn from_rosu(inner: GradualPerformance, ar: f64, cs: f64, hp: f64, od: f64) -> Self {
        Self {
            inner: Some(inner),
            ar_val: ar,
            cs_val: cs,
            hp_val: hp,
            od_val: od,
        }
    }
}

#[napi]
impl JsGradualPerformance {
    #[napi(constructor)]
    pub fn dummy_constructor() -> Result<Self> {
        Err(Error::new(Status::InvalidArg, "GradualPerformance cannot be constructed directly"))
    }

    #[napi(js_name = "nth")]
    pub fn nth(&mut self, state: ScoreState, n: u32) -> Option<JsPerformanceAttributes> {
        let ar = self.ar_val;
        let cs = self.cs_val;
        let hp = self.hp_val;
        let od = self.od_val;
        let rosu_state: RosuScoreState = state.clone().into();
        self.inner.as_mut()?.nth(rosu_state, n as usize).map(|attrs| JsPerformanceAttributes {
            inner: attrs,
            state_val: Some(state),
            ar_val: ar,
            cs_val: cs,
            hp_val: hp,
            od_val: od,
            clock_rate_val: 1.0,
            pp_aim_val: None,
            pp_speed_val: None,
            pp_acc_val: None,
            pp_reading_val: None,
            pp_fl_val: None,
            pp_total_val: None,
        })
    }

    #[napi(js_name = "next")]
    pub fn next(&mut self, state: ScoreState) -> Option<JsPerformanceAttributes> {
        let ar = self.ar_val;
        let cs = self.cs_val;
        let hp = self.hp_val;
        let od = self.od_val;
        let rosu_state: RosuScoreState = state.clone().into();
        self.inner.as_mut()?.next(rosu_state).map(|attrs| JsPerformanceAttributes {
            inner: attrs,
            state_val: Some(state),
            ar_val: ar,
            cs_val: cs,
            hp_val: hp,
            od_val: od,
            clock_rate_val: 1.0,
            pp_aim_val: None,
            pp_speed_val: None,
            pp_acc_val: None,
            pp_reading_val: None,
            pp_fl_val: None,
            pp_total_val: None,
        })
    }

    #[napi(getter, js_name = "nRemaining")]
    pub fn n_remaining(&self) -> u32 {
        self.inner.as_ref().map(|i| i.len() as u32).unwrap_or(0)
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}
