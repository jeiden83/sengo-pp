const { existsSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;

let nativeBinding = null;
let loadError = null;

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim();
      return require('fs').readFileSync(lddPath, 'utf8').includes('musl');
    } catch {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
  case 'win32':
    switch (arch) {
      case 'x64':
        try {
          nativeBinding = require('./sengo-pp.win32-x64-msvc.node');
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        try {
          nativeBinding = require('./sengo-pp.win32-arm64-msvc.node');
        } catch (e) {
          loadError = e;
        }
        break;
      case 'ia32':
        try {
          nativeBinding = require('./sengo-pp.win32-ia32-msvc.node');
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  case 'darwin':
    switch (arch) {
      case 'x64':
        try {
          nativeBinding = require('./sengo-pp.darwin-x64.node');
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        try {
          nativeBinding = require('./sengo-pp.darwin-arm64.node');
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          try {
            nativeBinding = require('./sengo-pp.linux-x64-musl.node');
          } catch (e) {
            loadError = e;
          }
        } else {
          try {
            nativeBinding = require('./sengo-pp.linux-x64-gnu.node');
          } catch (e) {
            loadError = e;
          }
        }
        break;
      case 'arm64':
        if (isMusl()) {
          try {
            nativeBinding = require('./sengo-pp.linux-arm64-musl.node');
          } catch (e) {
            loadError = e;
          }
        } else {
          try {
            nativeBinding = require('./sengo-pp.linux-arm64-gnu.node');
          } catch (e) {
            loadError = e;
          }
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError;
  }
  throw new Error(`Failed to load native binding for ${platform}-${arch}`);
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
