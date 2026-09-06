# 📜 Instrucciones y Protocolos de Desarrollo para Agentes en Sengo-PP

> [!IMPORTANT]
> **LECTURA OBLIGATORIA:** Si eres un agente de IA (trabajando directamente en este repositorio o interactuando remotamente desde el workspace de `sengo_bot` u otro proyecto), **DEBES LEER Y ACATAR ESTAS INSTRUCCIONES ANTES DE EDITAR CUALQUIER ARCHIVO**.
> No modifiques código en `sengo_pp` sin seguir rigurosamente las normas descritas aquí.

---

## 1. 🛡️ Normas de Edición y Protección del Código

1. **Paridad Exacta con osu!lazer C# (< 0.01% Delta):**
   - La verdad absoluta del cálculo de dificultad y PP es el motor oficial C# (`osu-tools` / Daemon).
   - Si tocas evaluadores o cálculo (`src/lazer_engine.rs`, `src/lazer_skills.rs`, `src/difficulty.rs`, `src/performance.rs`, `src/reading.rs`), debes verificar paridad con `node tools/tester.js <url_o_id> [mods]` y con `npm test`.
   - Nunca introduzcas regresiones ni reduzcas la precisión.

2. **Protección Absoluta de `index.d.ts` (Tipos TypeScript):**
   - Las definiciones de TypeScript en [index.d.ts](file:///./index.d.ts) están sincronizadas a mano para ofrecer soporte completo de autocompletado y tipado estricto a los consumidores de la librería.
   - **PROHIBIDO:** Ejecutar comandos de NAPI sin redirigir los tipos generados automáticamente (ya que NAPI sobreescribe `index.d.ts` con un archivo vacío).
   - Usa siempre `npm run build` o pasa `--dts binding.d.ts` para compilar localmente.
   - Antes de cualquier commit o release, comprueba que `index.d.ts` contenga todas las clases, interfaces y enums (`Beatmap`, `Performance`, `Difficulty`, `HitResultGenerator`, `GameMode`, etc.) y que su tamaño sea mayor a 1 KB.

3. **Pruebas obligatorias:**
   - Antes de dar por finalizada cualquier edición:
     ```powershell
     npm test
     ```
   - Las 22+ pruebas unitarias de la librería nativa deben pasar al 100%.

---

## 2. 📦 Protocolo Estricto de Release y Deploy a NPM

El despliegue a NPM y GitHub Releases es una operación crítica y delicada. Para publicar una nueva versión:

1. **Verificar colisión de versión en NPM:**
   - NPM **rechaza categóricamente (error 403 Forbidden)** publicar una versión que ya exista en el registro.
   - Antes de publicar, consulta siempre la última versión publicada:
     ```powershell
     npm view sengo-pp version
     ```
   - La versión que se publique **debe ser superior** (usualmente un bump de patch: `1.0.0` -> `1.0.1`).

2. **Sincronización de versiones:**
   - La nueva versión debe coincidir exactamente en dos archivos:
     - [package.json](file:///./package.json) (`"version": "X.Y.Z"`)
     - [Cargo.toml](file:///./Cargo.toml) (`version = "X.Y.Z"`)

3. **Verificación de empaquetado:**
   - Ejecutar `npm pack --dry-run --json` para asegurar que el tarball incluye:
     - `index.js`
     - `index.d.ts` (con su contenido completo)
     - `package.json`
     - `README.md`
     - `*.node` (binarios nativos)

4. **Publicación mediante Tag en GitHub Actions:**
   - El archivo [.github/workflows/release.yml](file:///./.github/workflows/release.yml) compila nativamente en 4 entornos:
     1. Windows x64 (`x86_64-pc-windows-msvc`)
     2. Linux x64 (`x86_64-unknown-linux-gnu`)
     3. macOS x64 Intel (`x86_64-apple-darwin`)
     4. macOS ARM64 Apple Silicon (`aarch64-apple-darwin`)
   - Reúne todos los binarios, crea los archivos `.tar.gz` y publica el paquete oficial en NPM utilizando el token `NPM_TOKEN`.
   - Este workflow **SOLO se dispara cuando se sube un tag de git** con formato `v*` (ej: `v1.0.1`).

---

## 3. 🤖 Automatización: Disparador "Ok subela"

Para evitar errores humanos y automatizar el flujo completo:

> ### ⚡ REGLA PARA EL AGENTE:
> Cuando el usuario diga **"Ok subela"**, **"subela"**, **"haz el deploy"**, **"publica a npm"** o cualquier instrucción equivalente:
> 
> **NO preguntes ni vaciles. Ejecuta inmediatamente el script automatizado:**
> ```powershell
> npm run release
> ```
> *(o `node tools/release.js`)*

### ¿Qué hace automáticamente `npm run release`?
1. Ejecuta `npm test` (si falla alguna prueba, se detiene de inmediato).
2. Verifica la integridad de `index.d.ts` (valida tamaño y firmas de tipos).
3. Consulta NPM vía `npm view sengo-pp version`.
4. Si la versión local ya existe en NPM, realiza auto-bump de versión patch automáticamente.
5. Sincroniza `package.json` y `Cargo.toml` con la versión correcta.
6. Valida el contenido del tarball con `npm pack --dry-run --json`.
7. Realiza el commit correspondiente (`chore(release): bump version to X.Y.Z`).
8. Hace `git push origin main`.
9. Crea el tag `vX.Y.Z` y hace `git push origin vX.Y.Z`.
10. Muestra el enlace directo a GitHub Actions para monitorear la compilación multi-plataforma y publicación.
