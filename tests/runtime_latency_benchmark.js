const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const sengo = require('../index.js');
const rosuPath = 'C:\\Users\\Jeiden\\Documents\\Programacion\\Javascript\\proyectos\\sengo_bot\\node_modules\\rosu-pp-js';
const rosu = require(rosuPath);
const csharpDllPath = path.resolve(__dirname, '../osu-tools/PerformanceCalculator/bin/Release/net8.0/PerformanceCalculator.dll');

const sampleMapPath = 'C:\\Users\\Jeiden\\Documents\\Programacion\\Javascript\\proyectos\\sengo_bot\\db\\local\\beatmap.osu\\1001305\\2096053.osu';
const mapBytes = fs.readFileSync(sampleMapPath);

console.log("===============================================================================");
console.log("   BENCHMARK DE RENDIMIENTO Y LATENCIA DE RUNTIME: SENGO-PP VS ROSU VS C#     ");
console.log("===============================================================================");

// ----------------------------------------------------
// 1. Benchmark de Parseo de Beatmap (1,000 iteraciones)
// ----------------------------------------------------
const PARSE_ITERATIONS = 1000;
console.log(`\n1. PARSEO DE BEATMAP (${PARSE_ITERATIONS} iteraciones):`);

// Sengo-PP (Rust Native)
const startSengoParse = performance.now();
for (let i = 0; i < PARSE_ITERATIONS; i++) {
    const map = new sengo.Beatmap(mapBytes);
}
const sengoParseTime = performance.now() - startSengoParse;
const sengoParseAvg = (sengoParseTime / PARSE_ITERATIONS);

// Rosu-pp-js (WASM)
const startRosuParse = performance.now();
for (let i = 0; i < PARSE_ITERATIONS; i++) {
    const map = new rosu.Beatmap(mapBytes);
    map.free();
}
const rosuParseTime = performance.now() - startRosuParse;
const rosuParseAvg = (rosuParseTime / PARSE_ITERATIONS);

console.log(`  • sengo-pp (Rust Native):   Total: ${sengoParseTime.toFixed(2)} ms | Promedio: ${sengoParseAvg.toFixed(4)} ms/mapa | ${(1000 / sengoParseAvg).toFixed(0)} maps/sec`);
console.log(`  • rosu-pp-js (WASM):        Total: ${rosuParseTime.toFixed(2)} ms | Promedio: ${rosuParseAvg.toFixed(4)} ms/mapa | ${(1000 / rosuParseAvg).toFixed(0)} maps/sec`);

// ----------------------------------------------------
// 2. Benchmark de Cálculo de PP (10,000 iteraciones sobre mapa cargado)
// ----------------------------------------------------
const CALC_ITERATIONS = 10000;
console.log(`\n2. CÁLCULO DE DIFFICULTY & PP (${CALC_ITERATIONS} iteraciones):`);

const sengoMap = new sengo.Beatmap(mapBytes);
const rosuMap = new rosu.Beatmap(mapBytes);

// Sengo-PP (Rust Native)
const startSengoCalc = performance.now();
let sengoLastPP = 0;
for (let i = 0; i < CALC_ITERATIONS; i++) {
    const perf = new sengo.Performance({ mods: 'HDDT', accuracy: 98.5, combo: 250, misses: 1 }).calculate(sengoMap);
    sengoLastPP = perf.pp;
}
const sengoCalcTime = performance.now() - startSengoCalc;
const sengoCalcAvg = (sengoCalcTime / CALC_ITERATIONS);

// Rosu-pp-js (WASM)
const startRosuCalc = performance.now();
let rosuLastPP = 0;
for (let i = 0; i < CALC_ITERATIONS; i++) {
    const perf = new rosu.Performance({ mods: 'HDDT', accuracy: 98.5, combo: 250, misses: 1 }).calculate(rosuMap);
    rosuLastPP = perf.pp;
    perf.free();
}
const rosuCalcTime = performance.now() - startRosuCalc;
const rosuCalcAvg = (rosuCalcTime / CALC_ITERATIONS);

console.log(`  • sengo-pp (Rust Native):   Total: ${sengoCalcTime.toFixed(2)} ms | Promedio: ${sengoCalcAvg.toFixed(4)} ms/cálculo | ${(1000 / sengoCalcAvg).toFixed(0)} calcs/sec`);
console.log(`  • rosu-pp-js (WASM):        Total: ${rosuCalcTime.toFixed(2)} ms | Promedio: ${rosuCalcAvg.toFixed(4)} ms/cálculo | ${(1000 / rosuCalcAvg).toFixed(0)} calcs/sec`);

// ----------------------------------------------------
// 3. Benchmark de C# Engine (.NET 8 Process)
// ----------------------------------------------------
console.log(`\n3. MOTOR OFICIAL C# (.NET 8):`);
const startCSharp = performance.now();
const csharpStdout = execSync(`dotnet "${csharpDllPath}" simulate osu "${sampleMapPath}" -j -m hd -m dt -a 98.5 --combo 250 -X 1`).toString();
const csharpTotalTime = performance.now() - startCSharp;
const startIdx = csharpStdout.indexOf('{');
const csharpResult = JSON.parse(csharpStdout.substring(startIdx));
const csharpLastPP = csharpResult.performance_attributes.pp;
const csharpStars = csharpResult.difficulty_attributes.star_rating;

console.log(`  • C# .NET 8 CLI (Subproceso): Total: ${csharpTotalTime.toFixed(2)} ms / cálculo`);

// ----------------------------------------------------
// 4. Comparativa de Desviación en Valores
// ----------------------------------------------------
console.log(`\n4. COMPARATIVA DE VALORES Y DESVIACIÓN (HDDT 98.5% 250x 1m):`);
console.log(`  • sengo-pp:    PP = ${sengoLastPP.toFixed(4)} PP`);
console.log(`  • rosu-pp-js:  PP = ${rosuLastPP.toFixed(4)} PP`);
console.log(`  • C# Lazer:    PP = ${csharpLastPP.toFixed(4)} PP`);

const diffSengoVsRosu = Math.abs(sengoLastPP - rosuLastPP);
const diffSengoVsCSharp = Math.abs(sengoLastPP - csharpLastPP);

console.log(`\n  -> Desviación sengo-pp vs rosu-pp-js: ${diffSengoVsRosu.toFixed(6)} PP (${((diffSengoVsRosu / rosuLastPP) * 100).toFixed(4)}%)`);
console.log(`  -> Desviación sengo-pp vs C# Lazer:   ${diffSengoVsCSharp.toFixed(6)} PP (${((diffSengoVsCSharp / csharpLastPP) * 100).toFixed(4)}%)`);

console.log("\n===============================================================================");
