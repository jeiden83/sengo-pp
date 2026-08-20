use napi_derive::napi;
pub use rosu_pp::model::mode::GameMode as RosuGameMode;

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum GameMode {
    Osu = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

impl From<GameMode> for RosuGameMode {
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Osu => RosuGameMode::Osu,
            GameMode::Taiko => RosuGameMode::Taiko,
            GameMode::Catch => RosuGameMode::Catch,
            GameMode::Mania => RosuGameMode::Mania,
        }
    }
}

impl From<RosuGameMode> for GameMode {
    fn from(mode: RosuGameMode) -> Self {
        match mode {
            RosuGameMode::Osu => GameMode::Osu,
            RosuGameMode::Taiko => GameMode::Taiko,
            RosuGameMode::Catch => GameMode::Catch,
            RosuGameMode::Mania => GameMode::Mania,
        }
    }
}

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum HitResultGenerator {
    Fast = 0,
    Closest = 1,
}

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum HitResultPriority {
    BestCase = 0,
    WorstCase = 1,
}
