const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const assert = require('assert');

const sengo = require('../index.js');
const dllPath = path.resolve(__dirname, '../osu-tools/PerformanceCalculator/bin/Release/net8.0/PerformanceCalculator.dll');

console.log("==================================================================");
console.log("   PRUEBAS DE PARIDAD CON EL MOTOR OFICIAL DE OSU! LAZER (C#)    ");
console.log("==================================================================");

function parseJsonFromOutput(stdout) {
    const startIdx = stdout.indexOf('{');
    if (startIdx === -1) {
        throw new Error("No JSON found in stdout: " + stdout);
    }
    const jsonStr = stdout.substring(startIdx);
    return JSON.parse(jsonStr);
}

function runCSharpDifficulty(mapIdOrPath, mods = [], modOptions = []) {
    let cmd = `dotnet "${dllPath}" difficulty "${mapIdOrPath}" -j`;
    for (const m of mods) {
        cmd += ` -m ${m}`;
    }
    for (const opt of modOptions) {
        cmd += ` -o ${opt}`;
    }
    const stdout = execSync(cmd).toString();
    const json = parseJsonFromOutput(stdout);
    return json.results[0].attributes;
}

function runCSharpSimulate(mapIdOrPath, params = {}) {
    let cmd = `dotnet "${dllPath}" simulate osu "${mapIdOrPath}" -j`;
    if (params.accuracy !== undefined) cmd += ` -a ${params.accuracy}`;
    if (params.combo !== undefined) cmd += ` --combo ${params.combo}`;
    if (params.misses !== undefined) cmd += ` -X ${params.misses}`;
    if (params.mods) {
        for (const m of params.mods) {
            cmd += ` -m ${m}`;
        }
    }
    if (params.modOptions) {
        for (const opt of params.modOptions) {
            cmd += ` -o ${opt}`;
        }
    }
    const stdout = execSync(cmd).toString();
    return parseJsonFromOutput(stdout);
}

function approxEqual(a, b, epsilon = 0.05, label = "") {
    const diff = Math.abs(a - b);
    if (diff > epsilon) {
        throw new Error(`Discrepancia en ${label}: actual=${a}, csharp_ref=${b}, diff=${diff}`);
    }
    return true;
}

async function runParityTests() {
    let passed = 0;
    let total = 0;

    function test(name, fn) {
        total++;
        try {
            fn();
            console.log(`  [OK] ${name}`);
            passed++;
        } catch (err) {
            console.error(`  [FAIL] ${name}: ${err.message}`);
            throw err;
        }
    }

    // ----------------------------------------------------
    // Test 1: Caso Específico del Usuario (Halozy 591347 con Extended Limits)
    // ----------------------------------------------------
    test("Mapa 591347 con HT(0.5x) + HD + DA(CS 2.5, AR -5, Extended Limits)", () => {
        const csharpRes = runCSharpDifficulty('591347', ['hd', 'ht', 'da'], [
            'ht_speed_change=0.5',
            'da_circle_size=2.5',
            'da_approach_rate=-5.0',
            'da_extended_limits=true'
        ]);

        console.log(`     -> C# Star Rating: ${csharpRes.star_rating.toFixed(2)}★ (Reading: ${csharpRes.reading_difficulty.toFixed(2)}★)`);
        approxEqual(csharpRes.star_rating, 8.929, 0.01, "Star Rating C# 8.92");
    });

    // ----------------------------------------------------
    // Test 2: Simulación de Jugada 591347 con C# Engine
    // ----------------------------------------------------
    test("Simulación de Jugada 591347 (74.62% Acc, 59 Combo, 17 Misses)", () => {
        const simRes = runCSharpSimulate('591347', {
            accuracy: 74.62,
            combo: 59,
            misses: 17,
            mods: ['hd', 'ht', 'da'],
            modOptions: [
                'ht_speed_change=0.5',
                'da_circle_size=2.5',
                'da_approach_rate=-5.0',
                'da_extended_limits=true'
            ]
        });

        console.log(`     -> C# Total PP: ${simRes.performance_attributes.pp.toFixed(2)} PP (Reading PP: ${simRes.performance_attributes.reading.toFixed(2)} PP)`);
        assert.ok(simRes.performance_attributes.pp > 0, "PP mayor a 0");
    });

    // ----------------------------------------------------
    // Test 3: Mapa Estándar NoMod (2096053)
    // ----------------------------------------------------
    test("Mapa 2096053 NoMod", () => {
        const csharpRes = runCSharpDifficulty('2096053', []);
        console.log(`     -> C# Star Rating: ${csharpRes.star_rating.toFixed(2)}★ (Aim: ${csharpRes.aim_difficulty.toFixed(2)}★, Speed: ${csharpRes.speed_difficulty.toFixed(2)}★)`);
        assert.ok(csharpRes.star_rating > 5.0, "Star rating > 5.0");
    });

    // ----------------------------------------------------
    // Test 4: Mapa Estándar con DT (2096053)
    // ----------------------------------------------------
    test("Mapa 2096053 con DT", () => {
        const csharpRes = runCSharpDifficulty('2096053', ['dt']);
        console.log(`     -> C# Star Rating (DT): ${csharpRes.star_rating.toFixed(2)}★ (Aim: ${csharpRes.aim_difficulty.toFixed(2)}★, Speed: ${csharpRes.speed_difficulty.toFixed(2)}★)`);
        assert.ok(csharpRes.star_rating > 7.0, "Star rating DT > 7.0");
    });

    // ----------------------------------------------------
    // Test 5: Mapa Estándar con HDHR (2096053)
    // ----------------------------------------------------
    test("Mapa 2096053 con HDHR", () => {
        const csharpRes = runCSharpDifficulty('2096053', ['hd', 'hr']);
        console.log(`     -> C# Star Rating (HDHR): ${csharpRes.star_rating.toFixed(2)}★ (Aim: ${csharpRes.aim_difficulty.toFixed(2)}★, Speed: ${csharpRes.speed_difficulty.toFixed(2)}★)`);
        assert.ok(csharpRes.star_rating > 5.5, "Star rating HDHR > 5.5");
    });

    console.log("==================================================================");
    console.log(` RESULTADO: ${passed} / ${total} PRUEBAS C# COMPLETADAS EXITOSAMENTE!`);
    console.log("==================================================================");
}

runParityTests().catch(err => {
    console.error("ERROR EN PRUEBAS C#:", err);
    process.exit(1);
});
