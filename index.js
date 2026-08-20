const { existsSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;

let nativeBinding = null;

if (platform === 'win32') {
  if (arch === 'x64') {
    const localFile = join(__dirname, 'sengo-pp.win32-x64-msvc.node');
    if (existsSync(localFile)) {
      nativeBinding = require(localFile);
    }
  }
}

if (!nativeBinding) {
  try {
    nativeBinding = require('./sengo-pp.win32-x64-msvc.node');
  } catch (e) {
    throw new Error(`Failed to load native binding for ${platform}-${arch}: ${e.message}`);
  }
}

const {
  Beatmap,
  Performance,
  Difficulty,
  BeatmapAttributesBuilder,
  GradualPerformance,
  JsGradualDifficulty,
  GradualDifficulty = JsGradualDifficulty,
  JsPerformanceAttributes,
  PerformanceAttributes = JsPerformanceAttributes,
  JsDifficultyAttributes,
  DifficultyAttributes = JsDifficultyAttributes,
  JsStrains,
  Strains = JsStrains,
  GameMode,
  HitResultGenerator,
  HitResultPriority
} = nativeBinding;

// Seamless polymorphic dispatcher for Performance.prototype.calculate
Performance.prototype.calculate = function (target) {
  if (!target) {
    throw new TypeError("Performance.calculate requires a Beatmap or DifficultyAttributes instance");
  }
  if (typeof target.bpm === 'number' || target instanceof Beatmap || target.nObjects !== undefined) {
    return this.calculateBeatmap(target);
  }
  if (typeof target.stars === 'number' || target instanceof DifficultyAttributes || target.maxCombo !== undefined) {
    return this.calculateAttributes(target);
  }
  return this.calculateBeatmap(target);
};

module.exports = {
  Beatmap,
  Performance,
  Difficulty,
  BeatmapAttributesBuilder,
  GradualPerformance,
  GradualDifficulty,
  PerformanceAttributes,
  DifficultyAttributes,
  Strains,
  GameMode,
  HitResultGenerator,
  HitResultPriority,
  default: {
    Beatmap,
    Performance,
    Difficulty,
    BeatmapAttributesBuilder,
    GradualPerformance,
    GradualDifficulty,
    PerformanceAttributes,
    DifficultyAttributes,
    Strains,
    GameMode,
    HitResultGenerator,
    HitResultPriority
  }
};
