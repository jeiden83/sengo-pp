use napi::bindgen_prelude::*;
use napi_derive::napi;
use rosu_pp::any::Strains;

#[napi(js_name = "Strains")]
pub struct JsStrains {
    pub(crate) section_len: f64,
    pub(crate) aim: Option<Vec<f64>>,
    pub(crate) aim_no_sliders: Option<Vec<f64>>,
    pub(crate) speed: Option<Vec<f64>>,
    pub(crate) flashlight: Option<Vec<f64>>,
    pub(crate) color: Option<Vec<f64>>,
    pub(crate) rhythm: Option<Vec<f64>>,
    pub(crate) stamina: Option<Vec<f64>>,
    pub(crate) reading: Option<Vec<f64>>,
    pub(crate) movement: Option<Vec<f64>>,
    pub(crate) strains: Option<Vec<f64>>,
}

#[napi]
impl JsStrains {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Err(Error::new(Status::InvalidArg, "Strains cannot be constructed directly"))
    }

    #[napi(getter, js_name = "sectionLength")]
    pub fn section_length(&self) -> f64 {
        self.section_len
    }

    #[napi(getter, js_name = "aim")]
    pub fn aim(&self) -> Option<Float64Array> {
        self.aim.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "aimNoSliders")]
    pub fn aim_no_sliders(&self) -> Option<Float64Array> {
        self.aim_no_sliders.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "speed")]
    pub fn speed(&self) -> Option<Float64Array> {
        self.speed.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "flashlight")]
    pub fn flashlight(&self) -> Option<Float64Array> {
        self.flashlight.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "color")]
    pub fn color(&self) -> Option<Float64Array> {
        self.color.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "rhythm")]
    pub fn rhythm(&self) -> Option<Float64Array> {
        self.rhythm.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "stamina")]
    pub fn stamina(&self) -> Option<Float64Array> {
        self.stamina.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "reading")]
    pub fn reading(&self) -> Option<Float64Array> {
        self.reading.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "movement")]
    pub fn movement(&self) -> Option<Float64Array> {
        self.movement.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(getter, js_name = "strains")]
    pub fn strains(&self) -> Option<Float64Array> {
        self.strains.as_ref().map(|v| Float64Array::new(v.clone()))
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }

    pub fn from_rosu(s: Strains) -> Self {
        s.into()
    }
}

impl From<Strains> for JsStrains {
    fn from(s: Strains) -> Self {
        let section_len = 400.0;
        match s {
            Strains::Osu(osu) => Self {
                section_len,
                aim: Some(osu.aim),
                aim_no_sliders: Some(osu.aim_no_sliders),
                speed: Some(osu.speed),
                flashlight: Some(osu.flashlight),
                color: None,
                rhythm: None,
                stamina: None,
                reading: None,
                movement: None,
                strains: None,
            },
            Strains::Taiko(taiko) => Self {
                section_len,
                aim: None,
                aim_no_sliders: None,
                speed: None,
                flashlight: None,
                color: Some(taiko.color),
                rhythm: Some(taiko.rhythm),
                stamina: Some(taiko.stamina),
                reading: Some(taiko.reading),
                movement: None,
                strains: None,
            },
            Strains::Catch(catch) => Self {
                section_len,
                aim: None,
                aim_no_sliders: None,
                speed: None,
                flashlight: None,
                color: None,
                rhythm: None,
                stamina: None,
                reading: None,
                movement: Some(catch.movement),
                strains: None,
            },
            Strains::Mania(mania) => Self {
                section_len,
                aim: None,
                aim_no_sliders: None,
                speed: None,
                flashlight: None,
                color: None,
                rhythm: None,
                stamina: None,
                reading: None,
                movement: None,
                strains: Some(mania.strains),
            },
        }
    }
}
