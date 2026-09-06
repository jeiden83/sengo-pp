const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.resolve(__dirname, '..');
const PACKAGE_JSON_PATH = path.join(ROOT_DIR, 'package.json');
const CARGO_TOML_PATH = path.join(ROOT_DIR, 'Cargo.toml');
const DTS_PATH = path.join(ROOT_DIR, 'index.d.ts');

function exec(cmd, options = {}) {
  return execSync(cmd, { cwd: ROOT_DIR, encoding: 'utf8', stdio: 'pipe', ...options });
}

function log(msg) {
  console.log(`\x1b[36m[sengo-release]\x1b[0m ${msg}`);
}

function error(msg) {
  console.error(`\x1b[31m[sengo-release ERROR]\x1b[0m ${msg}`);
}

function parseSemver(ver) {
  const parts = ver.replace(/^v/, '').split('.').map(Number);
  if (parts.length !== 3 || parts.some(isNaN)) {
    throw new Error(`Versión semver inválida: ${ver}`);
  }
  return { major: parts[0], minor: parts[1], patch: parts[2] };
}

function bumpVersion(current, type = 'patch') {
  const { major, minor, patch } = parseSemver(current);
  if (type === 'major') return `${major + 1}.0.0`;
  if (type === 'minor') return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}

function compareVersions(v1, v2) {
  const p1 = parseSemver(v1);
  const p2 = parseSemver(v2);
  if (p1.major !== p2.major) return p1.major - p2.major;
  if (p1.minor !== p2.minor) return p1.minor - p2.minor;
  return p1.patch - p2.patch;
}

async function main() {
  const args = process.argv.slice(2);
  const isDryRun = args.includes('--dry-run');
  const customVerArg = args.find(a => !a.startsWith('--'));

  log('Iniciando protocolo de release y deploy para sengo-pp...');

  // 1. Validar suite de pruebas nativa
  log('Ejecutando suite de pruebas (npm test)...');
  try {
    exec('npm test');
    log('✓ Todas las pruebas pasaron satisfactoriamente.');
  } catch (err) {
    error('Fallo en la suite de pruebas. Abortando release.');
    console.error(err.stdout || err.message);
    process.exit(1);
  }

  // 2. Validar index.d.ts
  log('Validando integridad de index.d.ts...');
  if (!fs.existsSync(DTS_PATH)) {
    error('index.d.ts no existe. Abortando.');
    process.exit(1);
  }
  const dtsContent = fs.readFileSync(DTS_PATH, 'utf8');
  if (dtsContent.length < 500 || !dtsContent.includes('export class Performance') || !dtsContent.includes('export enum GameMode')) {
    error('index.d.ts parece estar corrupto o vacío (tamaño < 500 bytes o faltan tipos clave). Abortando.');
    process.exit(1);
  }
  log('✓ index.d.ts verificado e intacto.');

  // 3. Consultar versión en NPM y package.json
  const pkg = JSON.parse(fs.readFileSync(PACKAGE_JSON_PATH, 'utf8'));
  let npmVersion = '0.0.0';
  try {
    npmVersion = exec('npm view sengo-pp version').trim();
  } catch (_) {
    log('No se pudo consultar versión en npm (¿paquete nuevo o sin conexión?). Asumiendo 0.0.0');
  }

  let targetVersion = pkg.version;

  if (customVerArg) {
    if (['major', 'minor', 'patch'].includes(customVerArg)) {
      targetVersion = bumpVersion(pkg.version, customVerArg);
    } else {
      parseSemver(customVerArg); // validate
      targetVersion = customVerArg.replace(/^v/, '');
    }
  } else {
    // Si la versión en package.json ya fue publicada en npm, auto-bump patch
    if (compareVersions(pkg.version, npmVersion) <= 0) {
      targetVersion = bumpVersion(npmVersion, 'patch');
      log(`La versión ${pkg.version} ya existe en NPM (última: ${npmVersion}). Auto-bump a ${targetVersion}`);
    }
  }

  if (compareVersions(targetVersion, npmVersion) <= 0) {
    error(`La versión destino ${targetVersion} debe ser mayor que la versión publicada en NPM (${npmVersion}).`);
    process.exit(1);
  }

  log(`Versión destino establecida: v${targetVersion}`);

  // 4. Actualizar package.json y Cargo.toml si cambió la versión
  if (pkg.version !== targetVersion) {
    pkg.version = targetVersion;
    fs.writeFileSync(PACKAGE_JSON_PATH, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
    log(`✓ Actualizado package.json -> ${targetVersion}`);
  }

  let cargoContent = fs.readFileSync(CARGO_TOML_PATH, 'utf8');
  const cargoVersionRegex = /(^\[package\][\s\S]*?version\s*=\s*")[^"]+(")/m;
  if (cargoVersionRegex.test(cargoContent)) {
    cargoContent = cargoContent.replace(cargoVersionRegex, `$1${targetVersion}$2`);
    fs.writeFileSync(CARGO_TOML_PATH, cargoContent, 'utf8');
    log(`✓ Actualizado Cargo.toml -> ${targetVersion}`);
  }

  // 5. Validar contenido del tarball con npm pack --dry-run --json
  log('Comprobando contenido del tarball con npm pack --dry-run --json...');
  try {
    const packJsonRaw = exec('npm pack --dry-run --json');
    const packJson = JSON.parse(packJsonRaw);
    const packedFiles = (packJson[0]?.files || []).map(f => f.path);
    if (!packedFiles.includes('index.d.ts') || !packedFiles.includes('index.js')) {
      error(`El paquete no contiene index.d.ts o index.js. Archivos presentes: ${packedFiles.join(', ')}`);
      process.exit(1);
    }
  } catch (err) {
    error(`Error al inspeccionar el paquete npm: ${err.message}`);
    process.exit(1);
  }
  log('✓ Tarball validado.');

  const tagName = `v${targetVersion}`;

  // Verificar si el tag ya existe
  const existingTags = exec('git tag -l').split(/\r?\n/);
  if (existingTags.includes(tagName)) {
    error(`El tag ${tagName} ya existe en git. Elimínalo o selecciona una nueva versión.`);
    process.exit(1);
  }

  if (isDryRun) {
    log(`[DRY-RUN] Simulación completa. Todo está verificado y listo.`);
    console.log(`Comandos que se ejecutarían:`);
    console.log(`  git add package.json Cargo.toml index.d.ts tools/release.js .gitignore`);
    console.log(`  git commit -m "chore(release): bump version to ${targetVersion} and prepare release"`);
    console.log(`  git push origin main`);
    console.log(`  git tag ${tagName}`);
    console.log(`  git push origin ${tagName}`);
    return;
  }

  // 6. Verificar git status y hacer commit si es necesario
  const gitStatus = exec('git status --porcelain').trim();
  if (gitStatus.length > 0) {
    log('Archivos con cambios detectados, preparando commit...');
    exec('git add package.json Cargo.toml index.d.ts tools/release.js .gitignore');
    try {
      exec(`git commit -m "chore(release): bump version to ${targetVersion} and prepare release"`);
      log(`✓ Commit realizado para v${targetVersion}`);
    } catch (_) {
      // Puede que ya esté commiteado
    }
  }

  // 7. Git push main
  log('Pusheando cambios a branch main...');
  exec('git push origin main', { stdio: 'inherit' });

  // 8. Crear y pushear tag
  log(`Creando tag ${tagName}...`);
  exec(`git tag ${tagName}`);
  log(`Pusheando tag ${tagName} a origin...`);
  exec(`git push origin ${tagName}`, { stdio: 'inherit' });

  console.log('\n================================================================');
  console.log(`  🚀 RELEASE v${targetVersion} DISPARADO CON ÉXITO A GITHUB ACTIONS`);
  console.log('================================================================');
  console.log(`- Monitorea el progreso del build y deploy en:`);
  console.log(`  https://github.com/jeiden83/sengo-pp/actions`);
  console.log(`- Se compilarán automáticamente los binarios para:`);
  console.log(`  • Windows (x86_64-pc-windows-msvc)`);
  console.log(`  • Linux (x86_64-unknown-linux-gnu)`);
  console.log(`  • macOS Intel (x86_64-apple-darwin)`);
  console.log(`  • macOS Apple Silicon (aarch64-apple-darwin)`);
  console.log(`- Se publicará automáticamente sengo-pp@${targetVersion} en NPM.`);
  console.log('================================================================\n');
}

main().catch(err => {
  error(err.message || err);
  process.exit(1);
});
