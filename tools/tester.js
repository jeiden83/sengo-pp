const { getBeatmap } = require('./beatmapFetcher');
const { calculateLazerDifficulty, calculateLazerPerformance, SengoLazerDaemon } = require('./lazerDaemon');
const sengo = require('../index');

let rosu = null;
try {
  rosu = require('rosu-pp-js');
} catch (_) {
  try {
    rosu = require('C:\\Users\\Jeiden\\Documents\\Programacion\\Javascript\\proyectos\\sengo_bot\\node_modules\\rosu-pp-js');
  } catch (e) {
    try {
      rosu = require('C:\\Users\\Jeiden\\node_modules\\rosu-pp-js');
    } catch (e2) {}
  }
}

/**
 * Normalizes mod string / array into standard formats
 */
function normalizeMods(modsInput) {
  if (!modsInput || modsInput === 'NM' || modsInput === 'NoMod') {
    return { str: '', list: [] };
  }
  if (Array.isArray(modsInput)) {
    const list = modsInput.map(m => m.toLowerCase());
    return { str: modsInput.join('').toUpperCase(), list };
  }
  const clean = modsInput.replace(/[^a-zA-Z]/g, '').toUpperCase();
  const list = [];
  for (let i = 0; i < clean.length; i += 2) {
    list.push(clean.substring(i, i + 2).toLowerCase());
  }
  return { str: clean, list };
}

/**
 * Executes a full comparison test across all 3 engines for a given beatmap and mods
 */
async function runComparisonTest(mapInput, modsInput = 'NM', accuracies = [100, 99, 98, 97, 95]) {
  const { mods, list: modList } = normalizeMods(modsInput);
  const modStr = modsInput || 'NM';

  // 1. Fetch / Cache Beatmap
  const mapData = await getBeatmap(mapInput);
  const { metadata, filePath, buffer, id } = mapData;

  const result = {
    beatmap: {
      id: id || 'Local File',
      title: metadata.title,
      artist: metadata.artist,
      version: metadata.version,
      mapper: metadata.creator,
      cs: metadata.cs,
      ar: metadata.ar,
      od: metadata.od,
      hp: metadata.hp,
      filePath
    },
    mods: modStr,
    sengo: {
      starRating: 0,
      aimSr: null,
      speedSr: null,
      readingSr: null,
      maxCombo: 0,
      accuracies: {},
      elapsedCalcMs: 0
    },
    rosu: {
      starRating: 0,
      maxCombo: 0,
      accuracies: {},
      elapsedCalcMs: 0
    },
    csharpLazer: {
      starRating: 0,
      aimSr: 0,
      speedSr: 0,
      readingSr: 0,
      flashlightSr: 0,
      maxCombo: 0,
      breakdownSS: {},
      accuracies: {},
      elapsedCalcMs: 0
    }
  };

  // ----------------------------------------------------
  // 2. SENGO-PP (Rust Native)
  // ----------------------------------------------------
  try {
    const sengoMap = new sengo.Beatmap(buffer);
    const startSengo = performance.now();
    const diffAttrs = new sengo.Difficulty({ mods: modStr }).calculate(sengoMap);
    result.sengo.starRating = diffAttrs.stars;
    result.sengo.maxCombo = diffAttrs.maxCombo;

    for (const acc of accuracies) {
      const perf = new sengo.Performance({ mods: modStr, accuracy: acc }).calculate(diffAttrs);
      result.sengo.accuracies[acc] = perf.pp;
      if (acc === 100) {
        result.sengo.aimSr = diffAttrs.aimStars;
        result.sengo.speedSr = diffAttrs.speedStars;
        result.sengo.breakdownSS = {
          aimPP: perf.ppAim,
          speedPP: perf.ppSpeed,
          accPP: perf.ppAcc,
          readingPP: perf.ppReading,
          flashlightPP: perf.ppFlashlight,
        };
      }
    }
    result.sengo.elapsedCalcMs = performance.now() - startSengo;
  } catch (err) {
    result.sengo.error = err.message;
  }

  // ----------------------------------------------------
  // 3. ROSU-PP-JS (WASM Legacy)
  // ----------------------------------------------------
  if (rosu) {
    try {
      const startRosu = performance.now();
      const rosuMap = new rosu.Beatmap(buffer);
      const diffOpts = modStr && modStr !== 'NM' ? { mods: modStr } : {};
      const rosuDiff = new rosu.Difficulty(diffOpts).calculate(rosuMap);
      result.rosu.starRating = rosuDiff.stars;
      result.rosu.maxCombo = rosuDiff.maxCombo;

      for (const acc of accuracies) {
        const perfOpts = modStr && modStr !== 'NM' ? { mods: modStr, accuracy: acc } : { accuracy: acc };
        const perf = new rosu.Performance(perfOpts).calculate(rosuDiff);
        result.rosu.accuracies[acc] = perf.pp;
        perf.free();
      }
      rosuDiff.free();
      rosuMap.free();
      result.rosu.elapsedCalcMs = performance.now() - startRosu;
    } catch (err) {
      result.rosu.error = err.message;
    }
  }

  // ----------------------------------------------------
  // 4. OSU-TOOLS (C# Official Lazer Engine via Daemon)
  // ----------------------------------------------------
  try {
    const { defaultDaemon } = require('./lazerDaemon');
    const startCSharp = performance.now();
    const csharpDiff = calculateLazerDifficulty(filePath, modList);
    result.csharpLazer.starRating = csharpDiff.star_rating;
    result.csharpLazer.aimSr = csharpDiff.aim_difficulty;
    result.csharpLazer.speedSr = csharpDiff.speed_difficulty;
    result.csharpLazer.readingSr = csharpDiff.reading_difficulty;
    result.csharpLazer.flashlightSr = csharpDiff.flashlight_difficulty;
    result.csharpLazer.maxCombo = csharpDiff.max_combo;

    for (const acc of accuracies) {
      const sim = calculateLazerPerformance(filePath, {
        mods: modList,
        accuracy: acc
      });
      const perf = sim.performance_attributes || {};
      result.csharpLazer.accuracies[acc] = perf.pp || 0;
      if (acc === 100) {
        result.csharpLazer.breakdownSS = {
          aimPP: perf.aim || 0,
          speedPP: perf.speed || 0,
          accPP: perf.accuracy || 0,
          readingPP: perf.reading || 0,
          flashlightPP: perf.flashlight || 0
        };
      }
    }
    result.csharpLazer.elapsedCalcMs = performance.now() - startCSharp;
  } catch (err) {
    result.csharpLazer.error = err.message;
  }

  return result;
}

/**
 * Prints formatted report to console
 */
function printReport(res) {
  const b = res.beatmap;
  console.log("===============================================================================");
  console.log(` 🎵 BEATMAP: ${b.artist} - ${b.title} [${b.version}]`);
  console.log(` 👤 Mapper: ${b.mapper} | ID: ${b.id} | Mods: ${res.mods}`);
  console.log(` ⚙️  Base Stats: CS ${b.cs} | AR ${b.ar} | OD ${b.od} | HP ${b.hp}`);
  console.log("===============================================================================");

  console.log("\n⭐ STAR RATING (DIFICULTAD):");
  console.log(`  • C# Lazer Oficial:   ${res.csharpLazer.starRating ? res.csharpLazer.starRating.toFixed(3) + '★' : 'N/A'}` +
              (res.csharpLazer.aimSr ? ` (Aim: ${res.csharpLazer.aimSr.toFixed(2)}★, Speed: ${res.csharpLazer.speedSr.toFixed(2)}★, Reading: ${res.csharpLazer.readingSr?.toFixed(2) || '0.00'}★)` : ''));
  
  let sengoSrDiffStr = '';
  if (res.sengo.starRating && res.csharpLazer.starRating) {
    const srDelta = res.sengo.starRating - res.csharpLazer.starRating;
    const srPct = (srDelta / res.csharpLazer.starRating) * 100;
    sengoSrDiffStr = ` (Delta: ${srDelta >= 0 ? '+' : ''}${srDelta.toFixed(3)}★ | ${srPct >= 0 ? '+' : ''}${srPct.toFixed(2)}%)`;
  }
  console.log(`  • Sengo-PP (Rust):    ${res.sengo.starRating ? res.sengo.starRating.toFixed(3) + '★' : 'N/A'}${sengoSrDiffStr}`);
  if (res.rosu.starRating) {
    console.log(`  • Rosu-PP (WASM):     ${res.rosu.starRating.toFixed(3)}★`);
  }
  console.log(`  • Max Combo:          ${res.csharpLazer.maxCombo || res.sengo.maxCombo}`);

  console.log("\n🏆 PERFORMANCE POINTS (PP):");
  console.log("  Acc %   | C# Lazer Oficial | Sengo-PP (Rust) | Rosu-PP (WASM) | Delta vs C# (PP / %)");
  console.log("  --------+------------------+-----------------+----------------+--------------------------");

  const accs = Object.keys(res.csharpLazer.accuracies).sort((a, b) => Number(b) - Number(a));
  for (const acc of accs) {
    const lazerPP = res.csharpLazer.accuracies[acc];
    const sengoPP = res.sengo.accuracies[acc];
    const rosuPP = res.rosu.accuracies[acc];
    const delta = (lazerPP !== undefined && sengoPP !== undefined) ? (sengoPP - lazerPP) : null;
    const pct = (delta !== null && lazerPP && lazerPP !== 0) ? (delta / lazerPP) * 100 : null;

    let deltaStr = 'N/A';
    if (delta !== null) {
      const signDelta = delta >= 0 ? '+' : '';
      const signPct = pct >= 0 ? '+' : '';
      deltaStr = `${signDelta}${delta.toFixed(2)} PP (${signPct}${pct.toFixed(2)}%)`;
    }

    console.log(`  ${acc.toString().padStart(5)}%  | ` +
                `${(lazerPP ? lazerPP.toFixed(2) + ' PP' : 'N/A').padStart(16)} | ` +
                `${(sengoPP ? sengoPP.toFixed(2) + ' PP' : 'N/A').padStart(15)} | ` +
                `${(rosuPP ? rosuPP.toFixed(2) + ' PP' : 'N/A').padStart(14)} | ` +
                `${deltaStr.padStart(24)}`);
  }

  if (res.csharpLazer.breakdownSS && res.csharpLazer.breakdownSS.aimPP !== undefined) {
    const bk = res.csharpLazer.breakdownSS;
    const sbk = res.sengo.breakdownSS || {};
    console.log("\n🔍 DESGLOSE DE ATRIBUTOS EN 100% SS:");
    console.log(`  • Aim PP:        C# ${bk.aimPP.toFixed(2)} PP | Sengo ${(sbk.aimPP || 0).toFixed(2)} PP (Δ: ${((sbk.aimPP || 0) - bk.aimPP).toFixed(2)})`);
    console.log(`  • Speed PP:      C# ${bk.speedPP.toFixed(2)} PP | Sengo ${(sbk.speedPP || 0).toFixed(2)} PP (Δ: ${((sbk.speedPP || 0) - bk.speedPP).toFixed(2)})`);
    console.log(`  • Accuracy PP:   C# ${bk.accPP.toFixed(2)} PP | Sengo ${(sbk.accPP || 0).toFixed(2)} PP (Δ: ${((sbk.accPP || 0) - bk.accPP).toFixed(2)})`);
    console.log(`  • Reading PP:    C# ${bk.readingPP?.toFixed(2) || '0.00'} PP | Sengo ${(sbk.readingPP || 0).toFixed(2)} PP (Δ: ${((sbk.readingPP || 0) - (bk.readingPP || 0)).toFixed(2)})`);
    if (bk.flashlightPP || sbk.flashlightPP) console.log(`  • Flashlight PP: C# ${bk.flashlightPP?.toFixed(2) || '0.00'} PP | Sengo ${(sbk.flashlightPP || 0).toFixed(2)} PP`);
  }

  console.log("\n⚡ TIEMPOS DE EJECUCIÓN (LATENCIA):");
  console.log(`  • Sengo-PP (Rust Nativo): ${res.sengo.elapsedCalcMs.toFixed(3)} ms  🚀 (Memoria Directa)`);
  if (res.rosu.elapsedCalcMs) console.log(`  • Rosu-PP (WASM):         ${res.rosu.elapsedCalcMs.toFixed(3)} ms`);
  console.log(`  • C# Lazer (.NET Engine): ${res.csharpLazer.elapsedCalcMs.toFixed(2)} ms`);
  console.log("===============================================================================\n");
}

// CLI Execution support
if (require.main === module) {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    console.log("Uso: node tools/tester.js <url_o_id> [mods] [acc1,acc2,...]");
    console.log("Ejemplo: node tools/tester.js https://osu.ppy.sh/beatmapsets/2093204#osu/4388676 NM");
    console.log("Ejemplo: node tools/tester.js 339055 DT 100,99,98,97,95");
    process.exit(0);
  }

  const mapInput = args[0];
  const modsInput = args[1] || 'NM';
  const accsInput = args[2] ? args[2].split(',').map(Number) : [100, 99, 98, 97, 95];

  runComparisonTest(mapInput, modsInput, accsInput)
    .then(report => {
      printReport(report);
      process.exit(0);
    })
    .catch(err => {
      console.error("Error ejecutando prueba:", err);
      process.exit(1);
    });
}

module.exports = {
  runComparisonTest,
  printReport
};
