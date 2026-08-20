use napi::bindgen_prelude::*;
use napi_derive::napi;
use rosu_pp::Beatmap;
use rosu_pp::model::hit_object::HitObjectKind;
use rosu_pp::GameMods;
use crate::enums::GameMode;
use crate::mods::parse_mods;

#[napi(js_name = "Beatmap")]
pub struct JsBeatmap {
    pub(crate) inner: Beatmap,
}

#[napi]
impl JsBeatmap {
    #[napi(constructor)]
    pub fn new(content: Buffer) -> Result<Self> {
        let inner = Beatmap::from_bytes(content.as_ref())
            .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to parse beatmap: {e}")))?;
        Ok(Self { inner })
    }

    #[napi(getter, js_name = "bpm")]
    pub fn bpm(&self) -> f64 {
        self.inner.bpm()
    }

    #[napi(getter, js_name = "nObjects")]
    pub fn n_objects(&self) -> u32 {
        self.inner.hit_objects.len() as u32
    }

    #[napi(getter, js_name = "nCircles")]
    pub fn n_circles(&self) -> u32 {
        self.inner.hit_objects.iter().filter(|h| matches!(h.kind, HitObjectKind::Circle)).count() as u32
    }

    #[napi(getter, js_name = "nSliders")]
    pub fn n_sliders(&self) -> u32 {
        self.inner.hit_objects.iter().filter(|h| matches!(h.kind, HitObjectKind::Slider { .. })).count() as u32
    }

    #[napi(getter, js_name = "nSpinners")]
    pub fn n_spinners(&self) -> u32 {
        self.inner.hit_objects.iter().filter(|h| matches!(h.kind, HitObjectKind::Spinner { .. })).count() as u32
    }

    #[napi(getter, js_name = "nHolds")]
    pub fn n_holds(&self) -> u32 {
        self.inner.hit_objects.iter().filter(|h| match h.kind {
            HitObjectKind::Circle | HitObjectKind::Slider { .. } | HitObjectKind::Spinner { .. } => false,
            _ => true,
        }).count() as u32
    }

    #[napi(getter, js_name = "ar")]
    pub fn ar(&self) -> f64 {
        self.inner.ar as f64
    }

    #[napi(getter, js_name = "cs")]
    pub fn cs(&self) -> f64 {
        self.inner.cs as f64
    }

    #[napi(getter, js_name = "hp")]
    pub fn hp(&self) -> f64 {
        self.inner.hp as f64
    }

    #[napi(getter, js_name = "od")]
    pub fn od(&self) -> f64 {
        self.inner.od as f64
    }

    #[napi(getter, js_name = "mode")]
    pub fn mode(&self) -> GameMode {
        self.inner.mode.into()
    }

    #[napi(getter, js_name = "isConvert")]
    pub fn is_convert(&self) -> bool {
        self.inner.is_convert
    }

    #[napi(js_name = "convert")]
    pub fn convert(&mut self, env: Env, mode: GameMode, mods: Option<Unknown>) -> Result<()> {
        let rosu_mode = mode.into();
        let parsed_intermode = match mods {
            Some(m) => parse_mods(&env, m)?,
            None => Default::default(),
        };
        let game_mods = GameMods::from(parsed_intermode);
        self.inner.convert_mut(rosu_mode, &game_mods)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to convert beatmap: {e}")))?;
        Ok(())
    }

    #[napi(js_name = "free")]
    pub fn free(&self) {
        // No-op for compatibility
    }
}
