const { spawn } = require('child_process');
const readline = require('readline');
const path = require('path');

const dotnetExe = 'C:\\Program Files\\dotnet\\dotnet.exe';
const dllPath = path.resolve(__dirname, '../osu-tools/PerformanceCalculator/bin/Release/net8.0/PerformanceCalculator.dll');
const sampleMapPath = 'C:\\Users\\Jeiden\\Documents\\Programacion\\Javascript\\proyectos\\sengo_bot\\db\\local\\beatmap.osu\\1001305\\2096053.osu';

class SengoLazerEngine {
    constructor() {
        this.process = null;
        this.rl = null;
        this.pendingRequests = [];
        this.isReady = false;
        this.readyPromise = null;
    }

    start() {
        if (this.readyPromise) return this.readyPromise;

        this.readyPromise = new Promise((resolve, reject) => {
            this.process = spawn(dotnetExe, [dllPath, 'daemon'], {
                stdio: ['pipe', 'pipe', 'inherit']
            });

            this.rl = readline.createInterface({
                input: this.process.stdout,
                crlfDelay: Infinity
            });

            this.rl.on('line', (line) => {
                line = line.trim();
                if (!line) return;

                if (!this.isReady) {
                    try {
                        const init = JSON.parse(line);
                        if (init.status === 'READY') {
                            this.isReady = true;
                            resolve();
                            return;
                        }
                    } catch (_) {}
                }

                if (this.pendingRequests.length > 0) {
                    const { resolveReq, rejectReq } = this.pendingRequests.shift();
                    try {
                        const parsed = JSON.parse(line);
                        if (parsed.success === false) {
                            rejectReq(new Error(parsed.error || 'Engine error'));
                        } else {
                            resolveReq(parsed);
                        }
                    } catch (err) {
                        rejectReq(err);
                    }
                }
            });

            this.process.on('error', reject);
        });

        return this.readyPromise;
    }

    async calculate(params) {
        await this.start();
        return new Promise((resolveReq, rejectReq) => {
            this.pendingRequests.push({ resolveReq, rejectReq });
            const jsonStr = JSON.stringify(params) + '\n';
            this.process.stdin.write(jsonStr);
        });
    }

    stop() {
        if (this.process) {
            this.process.stdin.write('EXIT\n');
            this.process = null;
        }
    }
}

async function testDaemonSpeed() {
    console.log("Iniciando Sengo Lazer Engine (Persistent Daemon)...");
    const engine = new SengoLazerEngine();
    await engine.start();
    console.log("Motor listo! Ejecutando prueba de calor (warmup)...");

    const warmup = await engine.calculate({
        beatmap: sampleMapPath,
        ruleset: "osu",
        mods: ["hd", "dt"],
        accuracy: 98.5,
        combo: 250,
        misses: 1
    });

    console.log(`Warmup finalizado: PP = ${warmup.pp.toFixed(2)} | SR = ${warmup.star_rating.toFixed(2)}★ | Tiempo C# interno: ${warmup.elapsed_ms.toFixed(2)} ms`);

    const ITERATIONS = 100;
    console.log(`\nEjecutando ${ITERATIONS} cálculos consecutivos en tiempo real...`);

    const start = performance.now();
    let lastRes = null;
    for (let i = 0; i < ITERATIONS; i++) {
        lastRes = await engine.calculate({
            beatmap: sampleMapPath,
            ruleset: "osu",
            mods: ["hd", "dt"],
            accuracy: 98.5,
            combo: 250,
            misses: 1
        });
    }
    const totalTime = performance.now() - start;
    const avgTime = totalTime / ITERATIONS;

    console.log(`\n=== RESULTADOS DE LATENCIA EN TIEMPO REAL ===`);
    console.log(`Total para ${ITERATIONS} cálculos: ${totalTime.toFixed(2)} ms`);
    console.log(`Promedio por cálculo: ${avgTime.toFixed(2)} ms / cálculo`);
    console.log(`Throughput: ${(1000 / avgTime).toFixed(0)} cálculos / segundo`);
    console.log(`Star Rating Exacto Lazer: ${lastRes.star_rating.toFixed(4)}★`);
    console.log(`PP Exacto Lazer (con Reading): ${lastRes.pp.toFixed(4)} PP`);
    console.log(`Reading PP: ${lastRes.reading_pp?.toFixed(4)} PP`);
    console.log(`Aim PP: ${lastRes.aim_pp?.toFixed(4)} PP`);
    console.log(`Speed PP: ${lastRes.speed_pp?.toFixed(4)} PP`);
    console.log(`Acc PP: ${lastRes.accuracy_pp?.toFixed(4)} PP`);

    engine.stop();
    process.exit(0);
}

testDaemonSpeed().catch(console.error);
