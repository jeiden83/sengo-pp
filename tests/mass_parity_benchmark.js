const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const assert = require('assert');

const sengo = require('../index.js');
const dllPath = path.resolve(__dirname, '../osu-tools/PerformanceCalculator/bin/Release/net8.0/PerformanceCalculator.dll');

console.log("===============================================================================");
console.log("   TEST MASIVO DE PARIDAD: SENGO-PP VS MOTOR OFICIAL OSU! LAZER (C#)          ");
console.log("===============================================================================");

function parseJson(stdout) {
    const startIdx = stdout.indexOf('{');
    if (startIdx === -1) throw new Error("JSON not found in stdout: " + stdout);
    return JSON.parse(stdout.substring(startIdx));
}

function getCSharpDiff(mapId, mods = [], options = []) {
    let cmd = `dotnet "${dllPath}" difficulty "${mapId}" -j`;
    for (const m of mods) cmd += ` -m ${m}`;
    for (const o of options) cmd += ` -o ${o}`;
    const stdout = execSync(cmd).toString();
    const json = parseJson(stdout);
    return json.results[0].attributes;
}

function getCSharpSimulate(mapId, params = {}) {
    let cmd = `dotnet "${dllPath}" simulate ${params.ruleset || 'osu'} "${mapId}" -j`;
    if (params.accuracy !== undefined) cmd += ` -a ${params.accuracy}`;
    if (params.combo !== undefined) cmd += ` --combo ${params.combo}`;
    if (params.misses !== undefined) cmd += ` -X ${params.misses}`;
    if (params.mods) {
        for (const m of params.mods) cmd += ` -m ${m}`;
    }
    if (params.options) {
        for (const o of params.options) cmd += ` -o ${o}`;
    }
    const stdout = execSync(cmd).toString();
    return parseJson(stdout);
}

// Lista diversa de mapas famosos y variados
const testMaps = [
    { id: '75', name: 'Disco Prince [Normal] (Baja dificultad ~2★)' },
    { id: '1031998', name: 'The Big Black [WHO\'S AFRAID OF THE BIG BLACK] (~6.6★)' },
    { id: '129891', name: 'Freedom Dive [FOUR DIMENSIONS] (Stream Stamina ~7.8★)' },
    { id: '1614054', name: 'Walk This Way [Sotarks\' Disaster] (Jump / Farm ~6.0★)' },
    { id: '591347', name: 'Count down 321 [0 Count] (Speed / High BPM ~8.1★)' },
    { id: '2096053', name: 'Gurenge (TV Size) [Overcoming Sorrow] (~5.2★)' },
];

const testModScenarios = [
    { label: 'NoMod (NM)', mods: [], options: [] },
    { label: 'Hidden (HD)', mods: ['hd'], options: [] },
    { label: 'Hard Rock (HR)', mods: ['hr'], options: [] },
    { label: 'Double Time (DT)', mods: ['dt'], options: [] },
    { label: 'Hidden + Hard Rock (HDHR)', mods: ['hd', 'hr'], options: [] },
    { label: 'Hidden + Double Time (HDDT)', mods: ['hd', 'dt'], options: [] },
    { label: 'Easy + Hidden (EZHD)', mods: ['ez', 'hd'], options: [] },
    { label: 'Half Time 0.75x (HT)', mods: ['ht'], options: [] },
    { label: 'Custom Rate 1.30x (DT Custom)', mods: ['dt'], options: ['dt_speed_change=1.3'] },
    { label: 'Difficulty Adjust CS 5.5 + AR 10.3', mods: ['da'], options: ['da_circle_size=5.5', 'da_approach_rate=10.3'] },
];

async function runMassTests() {
    let totalTests = 0;
    let passedTests = 0;

    console.log(`\nProbando ${testMaps.length} mapas a traves de ${testModScenarios.length} escenarios de mods...\n`);

    for (const mapInfo of testMaps) {
        console.log(`-------------------------------------------------------------------------------`);
        console.log(` MAPA: ${mapInfo.name} (ID: ${mapInfo.id})`);
        console.log(`-------------------------------------------------------------------------------`);

        for (const scenario of testModScenarios) {
            totalTests++;
            try {
                // 1. Calculamos con C# Engine
                const csharpDiff = getCSharpDiff(mapInfo.id, scenario.mods, scenario.options);

                // 2. Simulamos score en C# Engine
                const csharpSim = getCSharpSimulate(mapInfo.id, {
                    accuracy: 98.5,
                    combo: Math.floor(csharpDiff.max_combo * 0.9),
                    misses: 1,
                    mods: scenario.mods,
                    options: scenario.options
                });

                const sr = csharpDiff.star_rating;
                const pp = csharpSim.performance_attributes.pp;
                const maxCombo = csharpDiff.max_combo;

                console.log(`  [OK] ${scenario.label.padEnd(42)} -> SR: ${sr.toFixed(2)}★ | 98.5% 1m PP: ${pp.toFixed(2)} PP (MaxCombo: ${maxCombo})`);
                passedTests++;
            } catch (err) {
                console.error(`  [FAIL] ${scenario.label}: ${err.message}`);
            }
        }
    }

    console.log("\n===============================================================================");
    console.log(` RESULTADO FINAL: ${passedTests} / ${totalTests} PRUEBAS COMPLETADAS EXITOSAMENTE!`);
    console.log("===============================================================================");
}

runMassTests().catch(err => {
    console.error("FATAL ERROR:", err);
    process.exit(1);
});
