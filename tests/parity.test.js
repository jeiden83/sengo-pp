const fs = require('fs');
const path = require('path');
const assert = require('assert');

const sengo = require('../index.js');
const sampleMapPath = 'C:\\Users\\Jeiden\\Documents\\Programacion\\Javascript\\proyectos\\sengo_bot\\db\\local\\beatmap.osu\\1001305\\2096053.osu';

if (!fs.existsSync(sampleMapPath)) {
    console.error("Beatmap de prueba no encontrado en:", sampleMapPath);
    process.exit(1);
}

const mapBytes = fs.readFileSync(sampleMapPath);

console.log("=================================================");
console.log("   TEST DE PARIDAD Y FIABILIDAD: SENGO-PP       ");
console.log("=================================================");

function approxEqual(a, b, epsilon = 0.05, label = "") {
    const diff = Math.abs(a - b);
    if (diff > epsilon) {
        throw new Error(`Discrepancia en ${label}: sengo-pp=${a}, expected=${b}, diff=${diff}`);
    }
    return true;
}

async function runTests() {
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
    // Test 1: Carga de Beatmap y Metadatos
    // ----------------------------------------------------
    test("Beatmap metadata & properties", () => {
        const sengoMap = new sengo.Beatmap(mapBytes);

        assert.ok(sengoMap.bpm > 0, "bpm");
        assert.ok(sengoMap.ar > 0, "ar");
        assert.ok(sengoMap.cs > 0, "cs");
        assert.ok(sengoMap.hp > 0, "hp");
        assert.ok(sengoMap.od > 0, "od");
        assert.ok(sengoMap.nObjects > 0, "nObjects");
        assert.ok(sengoMap.nCircles > 0, "nCircles");
        assert.ok(sengoMap.nSliders > 0, "nSliders");
    });

    // ----------------------------------------------------
    // Test 2: Performance NoMod (NM)
    // ----------------------------------------------------
    test("Performance NM (NoMod)", () => {
        const sengoMap = new sengo.Beatmap(mapBytes);
        const sengoPerf = new sengo.Performance().calculate(sengoMap);

        assert.ok(sengoPerf.pp > 0, "pp > 0");
        assert.ok(sengoPerf.ppAim > 0, "ppAim > 0");
        assert.ok(sengoPerf.ppSpeed > 0, "ppSpeed > 0");
        assert.ok(sengoPerf.ppAccuracy > 0, "ppAccuracy > 0");
        assert.ok(sengoPerf.difficulty.maxCombo > 0, "maxCombo > 0");
    });

    // ----------------------------------------------------
    // Test 3: Performance con Mods Clásicos y Lazer (HD, HR, DT, FL, CL, etc.)
    // ----------------------------------------------------
    const modCases = ["HD", "HR", "DT", "HDDT", "EZ", "FL", "CL", "HDHRDT", "HDDTCL"];
    for (const mod of modCases) {
        test(`Performance con mod ${mod}`, () => {
            const sMap = new sengo.Beatmap(mapBytes);
            const sPerf = new sengo.Performance({ mods: mod }).calculate(sMap);

            assert.ok(sPerf.pp > 0, `pp (${mod}) > 0`);
            assert.ok(sPerf.ppAim > 0, `ppAim (${mod}) > 0`);
            assert.ok(sPerf.difficulty.maxCombo > 0, `maxCombo (${mod}) > 0`);
        });
    }

    // ----------------------------------------------------
    // Test 4: Performance con Mods en Formato Bitmask y Array
    // ----------------------------------------------------
    test("Performance con mods en bitmask (HD=8 + DT=64 = 72)", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const sPerf = new sengo.Performance({ mods: 72 }).calculate(sMap);

        assert.ok(sPerf.pp > 0, "pp bitmask > 0");
        assert.ok(sPerf.ppAim > 0, "ppAim bitmask > 0");
    });

    test("Performance con mods en Array de Objetos [{ acronym: 'HD' }, { acronym: 'DT' }]", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const modsArr = [{ acronym: 'HD' }, { acronym: 'DT' }];
        const sPerf = new sengo.Performance({ mods: modsArr }).calculate(sMap);

        assert.ok(sPerf.pp > 0, "pp array obj > 0");
        assert.ok(sPerf.ppAim > 0, "ppAim array obj > 0");
    });

    // ----------------------------------------------------
    // Test 5: Simulación de Jugada (Acc, Misses, Combos, Hits)
    // ----------------------------------------------------
    test("Performance Simulado (Acc 97.5%, 2 Misses, Combo 450)", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const params = {
            mods: "HDDT",
            accuracy: 97.5,
            misses: 2,
            combo: 450,
        };

        const sPerf = new sengo.Performance(params).calculate(sMap);

        assert.ok(sPerf.pp > 0, "pp simulado > 0");
        assert.ok(sPerf.ppAim > 0, "ppAim simulado > 0");
        assert.ok(sPerf.ppSpeed > 0, "ppSpeed simulado > 0");
    });

    // ----------------------------------------------------
    // Test 6: Performance calculando sobre DifficultyAttributes en Caché
    // ----------------------------------------------------
    test("Performance calculando sobre DifficultyAttributes en caché", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const sDiff = new sengo.Difficulty({ mods: "HR" }).calculate(sMap);
        const sPerf = new sengo.Performance({ accuracy: 99.0 }).calculate(sDiff);

        assert.ok(sPerf.pp > 0, "pp cached > 0");
        assert.ok(sPerf.ppAim > 0, "ppAim cached > 0");
    });

    // ----------------------------------------------------
    // Test 7: BeatmapAttributesBuilder
    // ----------------------------------------------------
    test("BeatmapAttributesBuilder con DT y custom CS/AR", () => {
        const sAttrs = new sengo.BeatmapAttributesBuilder({
            mods: "DT",
            ar: 9.0,
            cs: 4.0,
        }).build();

        approxEqual(sAttrs.ar, 10.33, 0.05, "attrs ar");
        approxEqual(sAttrs.cs, 4.0, 0.01, "attrs cs");
        approxEqual(sAttrs.clockRate, 1.5, 0.001, "attrs clockRate");
    });

    // ----------------------------------------------------
    // Test 8: Strains Generation (para gráficos de dificultad)
    // ----------------------------------------------------
    test("Strains generation para strainGraph.js", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const sDiff = new sengo.Difficulty({ mods: "HDDT" });
        const sStrains = sDiff.strains(sMap);

        assert.ok(sStrains.sectionLength > 0, "strain sectionLength");
        assert.ok(sStrains.aim instanceof Float64Array, "aim Float64Array");
        assert.ok(sStrains.speed instanceof Float64Array, "speed Float64Array");
        assert.ok(sStrains.aim.length > 0, "aim strains length");
        assert.ok(sStrains.speed.length > 0, "speed strains length");
    });

    // ----------------------------------------------------
    // Test 9: Gradual Calculations
    // ----------------------------------------------------
    test("GradualPerformance nth calculation", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const sDiff = new sengo.Difficulty();
        const mutsGP = sDiff.gradualPerformance(sMap);
        const sNth = mutsGP.nth({ maxCombo: 100, n300: 95, n100: 5, misses: 0 }, 100);

        assert.ok(sNth, "sNth not null");
        assert.ok(sNth.pp > 0, "gradual pp");
    });

    test("GradualDifficulty collect calculation", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        const sDiff = new sengo.Difficulty();
        const mutsGD = sDiff.gradualDifficulty(sMap);
        const sCollected = mutsGD.collect();

        assert.ok(sCollected.length > 0, "sCollected length");
        assert.ok(sCollected[0].stars > 0, "sCollected stars");
    });

    // ----------------------------------------------------
    // Test 10: Modos Taiko, Catch y Mania
    // ----------------------------------------------------
    test("Conversión de Beatmap a Taiko", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        sMap.convert(sengo.GameMode.Taiko);
        const sPerf = new sengo.Performance().calculate(sMap);
        assert.ok(sPerf.pp > 0, "taiko pp");
    });

    test("Conversión de Beatmap a Catch", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        sMap.convert(sengo.GameMode.Catch);
        const sPerf = new sengo.Performance().calculate(sMap);
        assert.ok(sPerf.pp > 0, "catch pp");
    });

    test("Conversión de Beatmap a Mania", () => {
        const sMap = new sengo.Beatmap(mapBytes);
        sMap.convert(sengo.GameMode.Mania);
        const sPerf = new sengo.Performance().calculate(sMap);
        assert.ok(sPerf.pp > 0, "mania pp");
    });

    console.log("=================================================");
    console.log(` RESULTADO: ${passed} / ${total} PRUEBAS SUPERADAS EXITOSAMENTE!`);
    console.log("=================================================");
}

runTests().catch(err => {
    console.error("FATAL ERROR EN PRUEBAS:", err);
    process.exit(1);
});
