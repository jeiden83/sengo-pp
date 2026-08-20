# 🛠️ Sengo-PP: Guía Técnica de Arquitectura y Mantenimiento de Reworks

Este documento sirve como bitácora técnica y guía paso a paso para el mantenimiento, depuración y actualización del motor `sengo-pp` ante futuros reworks de dificultad y PP de **osu!lazer**.

---

## 🏛️ 1. Arquitectura General del Motor

El motor está estructurado en 3 capas fundamentales:

```
                  ┌─────────────────────────────────────┐
                  │    index.js & index.d.ts (Node-API)  │
                  └──────────────────┬──────────────────┘
                                     │
           ┌─────────────────────────┴─────────────────────────┐
           ▼                                                   ▼
┌───────────────────────────────┐               ┌───────────────────────────────┐
│   src/difficulty.rs           │               │   src/performance.rs          │
│   - Star Rating & Skills      │               │   - PP Calculator             │
│   - Attributes Builder        │               │   - Deviations & Acc Scaling  │
└──────────────┬────────────────┘               └──────────────┬────────────────┘
               │                                               │
               └───────────────────────┬───────────────────────┘
                                       │
                                       ▼
                     ┌───────────────────────────────────┐
                     │   src/lazer_engine.rs             │
                     │   - Preprocessing de HitObjects   │
                     │   - PathApproximator (Bézier/Arc) │
                     │   - Evaluadores de Dificultad     │
                     │   - Stack Leniency                │
                     └─────────────────┬─────────────────┘
                                       │
                                       ▼
                     ┌───────────────────────────────────┐
                     │   src/lazer_skills.rs             │
                     │   - LazerAimSkill                 │
                     │   - LazerSpeedSkill               │
                     │   - LazerReadingSkill             │
                     │   - LazerFlashlightSkill          │
                     │   - VariableLengthStrainSkill     │
                     └───────────────────────────────────┘
```

---

## 📐 2. Fórmulas Clave y Parámetros Oficiales de Lazer

### A. Ventanas de Hit (Hit Windows & OD)
En osu!lazer, las ventanas de impacto incorporan un offset constante de `-0.5ms`:
* **Great (300):** $\text{greatWindow} = \frac{79.5 - 6.0 \times \text{OD}}{\text{ClockRate}}$
* **Ok (100):** $\text{okWindow} = \frac{139.5 - 8.0 \times \text{OD}}{\text{ClockRate}}$
* **Meh (50):** $\text{mehWindow} = \frac{199.5 - 10.0 \times \text{OD}}{\text{ClockRate}}$
* **OverallDifficulty:** $\text{overallDifficulty} = \frac{79.5 - \text{greatWindow}}{6.0}$

### B. Aproximación de Curvas (`PathApproximator`)
* **Curvas Bézier:** Subdivisión adaptativa de de Casteljau por árbol DFS con tolerancia subpíxel:
  $$\text{BEZIER\_TOLERANCE} = 0.25\text{px} \implies \text{TOLERANCE\_SQ} = 0.25^2 \times 4 = 0.25$$
* **Arcos Circulares:** Paso angular con tolerancia $0.1\text{px}$:
  $$d\theta = 2.0 \times \arccos\left(1.0 - \min\left(1.0, \frac{0.1}{r}\right)\right)$$
* **Sliders & Ticks:**
  - $\text{SpanCount} = \text{RepeatCount} + 1$
  - Intervalo de tick: $\text{tickInterval} = \max\left(1.0, \frac{\text{PixelsPerBeat}}{\text{SliderTickRate}}\right)$
  - Los ticks se generan dentro del rango: $d \le \text{PixelLength} - \frac{\text{tickInterval}}{8.0}$

### C. Habilidades y Escalamiento de Dificultad
* **Aim:**
  $$\text{SnapAim} = \text{SnapAimEvaluator} \times 70.9$$
  $$\text{AgilityAim} = \text{AgilityEvaluator} \times 2.35$$
  $$\text{FlowAim} = \text{FlowAimEvaluator} \times 242.0$$
  $$\text{AimTotal} = \text{Norm}_{1.2}(\text{SnapAim}, \text{AgilityAim}) \times \text{logistic} + \text{FlowAim} \times (1 - \text{logistic})$$
  $$\text{AimDifficultyRating} = (\text{DifficultyValue})^{0.63} \times 0.02275$$
* **Speed:**
  $$\text{SpeedDifficultyRating} = \sqrt{\text{DifficultyValue}} \times 0.0675$$
* **Reading:**
  $$\text{ReadingDifficultyRating} = \sqrt{\text{DifficultyValue}} \times 0.0675$$
* **Flashlight:**
  $$\text{FlashlightDifficultyRating} = \sqrt{\text{DifficultyValue}} \times 0.0675$$

### D. Performance Points (PP)
* **Aim PP:** $4.0 \times (\text{AimDifficulty})^3 \times \text{LengthBonus} \times \text{Acc}$
* **Speed PP:** $4.0 \times (\text{SpeedDifficulty})^3 \times \text{LengthBonus} \times \text{SpeedHighNerf} \times \text{Acc}$
* **Accuracy PP:** $(\text{AccValue})^{1.5} \times \text{LengthBonus}$
* **Reading PP:** $4.0 \times (\text{ReadingDifficulty})^3 \times \text{LengthBonus}$
* **Total PP:**
  $$\text{Total PP} = \left(\text{AimPP}^{1.1} + \text{SpeedPP}^{1.1} + \text{AccPP}^{1.1} + \text{ReadingPP}^{1.1} + \text{FlashlightPP}^{1.1}\right)^{1 / 1.1} \times 1.12$$

---

## 🔄 3. Pasos a Seguir ante un Nuevo Rework de Lazer

Cuando el equipo de osu! lance un nuevo rework o cambio en las fórmulas de dificultad:

### Paso 1: Actualizar y compilar `osu-tools`
```powershell
cd osu-tools
git pull origin master
dotnet build -c Release
cd ..
```

### Paso 2: Ejecutar el tester contra el Daemon oficial
Prueba varios mapas representativos para detectar qué habilidad diverge:
```powershell
node tools/tester.js 1625858 EZHD 100,99,98
node tools/tester.js 129891 HDHR 100,99,98
node tools/tester.js 131891 HDDT 100,99,98
```

### Paso 3: Identificar la habilidad o evaluador que cambió
1. Revisa si la divergencia está en **Aim**, **Speed**, **Accuracy**, **Reading** o **Flashlight**.
2. Revisa el commit diff en el repositorio `ppy/osu` o descompila las clases actualizadas en `csharp_sources/`.

### Paso 4: Ajustar en Rust y Validar
1. Edita el archivo correspondiente (`src/lazer_skills.rs`, `src/lazer_engine.rs` o `src/difficulty.rs`).
2. Compila el motor en Release:
   ```powershell
   npm run build
   ```
3. Ejecuta la suite de paridad masiva:
   ```powershell
   npm run test:mass
   npm run test:csharp
   ```
4. Asegúrate de que **todos los 60 escenarios pasen con $< 0.01\%$ de delta**.

---

## 💈 Principio Ponytail (Reglas de Desarrollo)
1. **La solución más simple y directa:** Evitar crear abstracciones innecesarias o parches específicos por mapa.
2. **Causa Raíz:** Siempre arreglar el evaluador matemático en lugar de colocar factores de corrección artificiales.
3. **Cero Regresiones:** Ningún cambio para arreglar un mod (ej. EZ) debe empeorar la paridad en otros mods (ej. DT, HR, NM).
