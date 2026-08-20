const { existsSync } = require('fs');
const { join, resolve } = require('path');
const { spawn, execSync } = require('child_process');
const readline = require('readline');

const csharpDllPath = resolve(__dirname, '../osu-tools/PerformanceCalculator/bin/Release/net8.0/PerformanceCalculator.dll');
const dotnetExe = 'C:\\Program Files\\dotnet\\dotnet.exe';

class SengoLazerDaemon {
  constructor(dllPath = csharpDllPath) {
    this.dllPath = dllPath;
    this.process = null;
    this.rl = null;
    this.pendingRequests = [];
    this.isReady = false;
    this.readyPromise = null;
  }

  start() {
    if (this.readyPromise) return this.readyPromise;

    this.readyPromise = new Promise((resolvePromise, reject) => {
      if (!existsSync(this.dllPath)) {
        return reject(new Error(`PerformanceCalculator DLL not found at: ${this.dllPath}. Make sure to build osu-tools with 'dotnet build -c Release'.`));
      }

      const execBin = existsSync(dotnetExe) ? dotnetExe : 'dotnet';
      this.process = spawn(execBin, [this.dllPath, 'daemon'], {
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
              resolvePromise();
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

      this.process.on('error', (err) => {
        this.isReady = false;
        this.readyPromise = null;
        reject(err);
      });

      this.process.on('exit', () => {
        this.isReady = false;
        this.readyPromise = null;
      });
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
      try {
        this.process.stdin.write('EXIT\n');
      } catch (_) {}
      this.process = null;
      this.isReady = false;
      this.readyPromise = null;
    }
  }
}

const defaultDaemon = new SengoLazerDaemon();

function parseJsonFromOutput(stdout) {
  const startIdx = stdout.indexOf('{');
  if (startIdx === -1) {
    throw new Error("No JSON found in output: " + stdout);
  }
  return JSON.parse(stdout.substring(startIdx));
}

function calculateLazerDifficulty(mapIdOrPath, mods = [], modOptions = []) {
  if (!existsSync(csharpDllPath)) {
    throw new Error(`PerformanceCalculator DLL not found at: ${csharpDllPath}`);
  }
  let cmd = `dotnet "${csharpDllPath}" difficulty "${mapIdOrPath}" -j`;
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

function calculateLazerPerformance(mapIdOrPath, params = {}) {
  if (!existsSync(csharpDllPath)) {
    throw new Error(`PerformanceCalculator DLL not found at: ${csharpDllPath}`);
  }
  let cmd = `dotnet "${csharpDllPath}" simulate ${params.ruleset || 'osu'} "${mapIdOrPath}" -j`;
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

async function calculateExactLazerAsync(params) {
  return defaultDaemon.calculate(params);
}

module.exports = {
  csharpDllPath,
  SengoLazerDaemon,
  defaultDaemon,
  calculateLazerDifficulty,
  calculateLazerPerformance,
  calculateExactLazerAsync
};
