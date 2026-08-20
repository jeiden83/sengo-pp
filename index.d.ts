export enum GameMode {
  Osu = 0,
  Taiko = 1,
  Catch = 2,
  Mania = 3,
}

export enum HitResultPriority {
  Fastest = 0,
  Normal = 1,
  Slowest = 2,
}

export interface ScoreState {
  maxCombo?: number;
  osuLargeTickHits?: number;
  osuSmallTickHits?: number;
  sliderEndHits?: number;
  nGeki?: number;
  nKatu?: number;
  n300?: number;
  n100?: number;
  n50?: number;
  misses?: number;
  legacyTotalScore?: number;
}

export interface PerformanceOptions {
  mods?: string | number | any[];
  accuracy?: number;
  combo?: number;
  misses?: number;
  n300?: number;
  n100?: number;
  n50?: number;
  nKatu?: number;
  nGeki?: number;
  sliderEndHits?: number;
  smallTickHits?: number;
  largeTickHits?: number;
  clockRate?: number;
  ar?: number;
  cs?: number;
  hp?: number;
  od?: number;
  passedObjects?: number;
  hitresultPriority?: HitResultPriority;
}

export interface DifficultyOptions {
  mods?: string | number | any[];
  clockRate?: number;
  ar?: number;
  cs?: number;
  hp?: number;
  od?: number;
  hardrockOffsets?: boolean;
}

export class Beatmap {
  constructor(content: Buffer | Uint8Array);
  bpm: number;
  ar: number;
  cs: number;
  hp: number;
  od: number;
  sliderMultiplier: number;
  sliderTickRate: number;
  mode: number;
  isConvert: boolean;
  nObjects: number;
  nCircles: number;
  nSliders: number;
  nSpinners: number;
  nBreaks: number;
  nHolds: number;
  convert(mode: GameMode, mods?: string | number | any[]): Beatmap;
}

export class DifficultyAttributes {
  mode: GameMode;
  stars: number;
  maxCombo: number;
  ar: number;
  cs: number;
  hp: number;
  od: number;
  clockRate: number;
  aim?: number;
  speed?: number;
  reading?: number;
  flashlight?: number;
  sliderFactor?: number;
  aimDifficultSliders?: number;
  speedNotes?: number;
  readingDifficultNotes?: number;
  aimDifficultStrains?: number;
  speedDifficultStrains?: number;
  aimTopWeightedSliderFactor?: number;
  speedTopWeightedSliderFactor?: number;
}

export class PerformanceAttributes {
  pp: number;
  ppAim?: number;
  ppSpeed?: number;
  ppAcc?: number;
  ppReading?: number;
  ppFlashlight?: number;
  effectiveMissCount?: number;
  speedDeviation?: number;
  difficultyAttributes?: DifficultyAttributes;
  state?: ScoreState;
}

export class Difficulty {
  constructor(options?: DifficultyOptions);
  mods(mods: string | number | any[]): this;
  clockRate(rate: number): this;
  ar(ar: number): this;
  cs(cs: number): this;
  hp(hp: number): this;
  od(od: number): this;
  hardrockOffsets(enabled: boolean): this;
  calculate(beatmap: Beatmap): DifficultyAttributes;
}

export class Performance {
  constructor(options?: PerformanceOptions);
  mods(mods: string | number | any[]): this;
  accuracy(acc: number): this;
  combo(combo: number): this;
  misses(misses: number): this;
  n300(n300: number): this;
  n100(n100: number): this;
  n50(n50: number): this;
  nKatu(nKatu: number): this;
  nGeki(nGeki: number): this;
  sliderEndHits(hits: number): this;
  smallTickHits(hits: number): this;
  largeTickHits(hits: number): this;
  clockRate(rate: number): this;
  ar(ar: number): this;
  cs(cs: number): this;
  hp(hp: number): this;
  od(od: number): this;
  passedObjects(n: number): this;
  hitresultPriority(priority: HitResultPriority): this;
  calculate(target: Beatmap | DifficultyAttributes): PerformanceAttributes;
}

export class BeatmapAttributesBuilder {
  constructor(beatmap?: Beatmap);
  map(beatmap: Beatmap): this;
  mods(mods: string | number | any[]): this;
  clockRate(rate: number): this;
  ar(ar: number): this;
  cs(cs: number): this;
  hp(hp: number): this;
  od(od: number): this;
  build(): any;
}

export class Strains {
  skills: any[];
  mode: GameMode;
  sectionLength: number;
}

export class GradualDifficulty {
  constructor(beatmap: Beatmap, options?: DifficultyOptions);
  next(): DifficultyAttributes | null;
  nth(n: number): DifficultyAttributes | null;
  collect(): DifficultyAttributes[];
}

export class GradualPerformance {
  constructor(beatmap: Beatmap, options?: DifficultyOptions);
  next(state?: ScoreState): PerformanceAttributes | null;
  nth(state: ScoreState, n: number): PerformanceAttributes | null;
}
