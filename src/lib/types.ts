// Mirrors the Rust types in src-tauri/src/model.rs and db.rs (camelCase via serde).

export type Source = "none" | "lmu" | "demo";
export type SessionKind = "unknown" | "testDay" | "practice" | "qualifying" | "warmup" | "race";

export interface Penalty {
  kind: "stopGo" | "driveThrough" | "other";
  /** Stationary time for a stop/go, 0 otherwise. */
  seconds: number;
  reason: string;
}

export interface Weather {
  ambientTemp: number;
  trackTemp: number;
  raining: number;
  darkCloud: number;
  cloudCoverage: number;
  windSpeed: number;
  windDirection: number;
  minWetness: number;
  maxWetness: number;
  avgWetness: number;
  gripLevel: number;
}

export interface Session {
  track: string;
  trackLength: number;
  code: number;
  kind: SessionKind;
  phase: number;
  currentEt: number;
  endEt: number;
  timeRemaining: number;
  maxLaps: number;
  timeOfDay: number;
  yellowFlagState: number;
  /** Local yellow per sector, S1..S3. */
  sectorYellow: [boolean, boolean, boolean];
  sectorFlagsRaw: [number, number, number];
  startLight: number;
  numRedLights: number;
  playerName: string;
  weather: Weather;
}

export interface Wheel {
  surfaceTemp: [number, number, number];
  innerTemp: [number, number, number];
  carcassTemp: number;
  optimalTemp: number;
  pressure: number;
  wear: number;
  brakeTemp: number;
  rideHeight: number;
  flat: boolean;
  detached: boolean;
  surfaceType: number;
}

export interface Player {
  id: number;
  vehicleName: string;
  vehicleModel: string;
  vehicleClass: string;
  elapsedTime: number;
  lapNumber: number;
  lapStartEt: number;
  lapDist: number;
  sector: number;
  inPits: boolean;
  speed: number;
  gear: number;
  maxGears: number;
  rpm: number;
  maxRpm: number;
  throttle: number;
  brake: number;
  clutch: number;
  steering: number;
  rawThrottle: number;
  rawBrake: number;
  rawSteering: number;
  steeringRangeDeg: number;
  gLat: number;
  gLong: number;
  pos: [number, number, number];
  yaw: number;
  fuel: number;
  fuelCapacity: number;
  virtualEnergy: number;
  battery: number;
  regenKw: number;
  electricMotorState: number;
  waterTemp: number;
  oilTemp: number;
  turboBoost: number;
  deltaBest: number;
  lapInvalidated: boolean;
  trackLimitsSteps: number;
  trackLimitsStepsPerPenalty: number;
  trackLimitsStepsPerPoint: number;
  liftAndCoast: number;
  tc: number;
  tcMax: number;
  tcSlip: number;
  tcCut: number;
  abs: number;
  absMax: number;
  tcActive: boolean;
  absActive: boolean;
  motorMap: number;
  motorMapMax: number;
  migration: number;
  frontArb: number;
  rearArb: number;
  brakeBiasFront: number;
  speedLimiter: boolean;
  headlights: boolean;
  wipers: number;
  /** 0 = off, 1 = ignition on, 2 = ignition + starter. */
  ignition: number;
  antiStall: boolean;
  /** Force feedback output, -1..1 (|1| = clipping). */
  ffb: number;
  overheating: boolean;
  detached: boolean;
  dents: number[];
  damage: Damage | null;
  lastImpactMagnitude: number;
  lastImpactEt: number;
  gapAhead: number;
  gapBehind: number;
  frontCompound: string;
  rearCompound: string;
  wheels: [Wheel, Wheel, Wheel, Wheel];
}

/** Repair-screen fractions: 0 = no damage; suspension order FL, FR, RL, RR. */
export interface Damage {
  aero: number;
  suspension: [number, number, number, number];
}

export interface Vehicle {
  id: number;
  place: number;
  classPlace: number;
  driver: string;
  vehicle: string;
  team: string;
  class: string;
  isPlayer: boolean;
  totalLaps: number;
  lapDist: number;
  sector: number;
  bestLap: number;
  lastLap: number;
  bestS1: number;
  bestS2: number;
  lastS1: number;
  lastS2: number;
  curS1: number;
  curS2: number;
  timeBehindNext: number;
  lapsBehindNext: number;
  timeBehindLeader: number;
  lapsBehindLeader: number;
  timeIntoLap: number;
  estimatedLapTime: number;
  inPits: boolean;
  pitState: number;
  inGarage: boolean;
  numPitstops: number;
  numPenalties: number;
  /** Unserved penalties the game logged, oldest first; may be shorter than numPenalties. */
  penalties: Penalty[];
  finishStatus: number;
  flag: number;
  fuelFraction: number;
  control: number;
  speed: number;
  pos: [number, number, number];
  /** Race number, e.g. "007". */
  number: string;
  model: string;
  tyreCompound: string;
  /** Remaining tread 0..1 (1 = new), null when unknown. */
  tyreWear: number | null;
  /** Laps on the current set. */
  tyreLaps: number;
  /** Remaining virtual energy 0..1, null when the car has none or it is unknown. */
  virtualEnergy: number | null;
}

export interface Timing {
  currentLapTime: number;
  lastLapTime: number;
  lastLapValid: boolean;
  sessionBest: number;
  personalBest: number;
  personalBestSectors: [number, number, number];
  sessionBestSectors: [number, number, number];
  lastSectors: [number, number, number];
  currentSectors: [number, number, number];
  deltaPb: number | null;
  predictedLap: number | null;
  stintLaps: number;
  stintTime: number;
}

export interface Strategy {
  fuelPerLap: number;
  vePerLap: number;
  lapsOfFuel: number;
  lapsOfVe: number;
  lapsRemaining: number;
  fuelToFinish: number;
  veToFinish: number;
  avgLapTime: number;
  samples: number;
}

export interface Snapshot {
  source: Source;
  connected: boolean;
  inRealtime: boolean;
  frame: number;
  gameVersion: number;
  session: Session;
  player: Player | null;
  vehicles: Vehicle[];
  timing: Timing;
  strategy: Strategy;
}

export interface Status {
  demo: boolean;
  editMode: boolean;
  overlayVisible: boolean;
  /** LMU window has focus (always false when the game is not running). */
  gameFocused: boolean;
  dataDir: string;
}

/** A connected display the overlay can cover. */
export interface MonitorInfo {
  /** Id stored in `general.monitor`. */
  name: string;
  width: number;
  height: number;
  x: number;
  y: number;
  primary: boolean;
}

export interface SessionSummary {
  id: number;
  startedAt: number;
  source: string;
  track: string;
  trackLength: number;
  sessionType: string;
  car: string;
  vehicleName: string;
  carClass: string;
  driver: string;
  /** LMU build number; null for demo data and older sessions. */
  gameVersion: number | null;
  lapCount: number;
  validLaps: number;
  bestLap: number | null;
}

export interface LapRecord {
  id: number;
  sessionId: number;
  lapNumber: number;
  lapTime: number;
  s1: number | null;
  s2: number | null;
  s3: number | null;
  valid: boolean;
  outLap: boolean;
  inLap: boolean;
  fuelUsed: number | null;
  veUsed: number | null;
  maxSpeed: number;
  track: string;
  car: string;
  carClass: string;
  recordedAt: number;
  sessionType: string;
}

export interface LapTrace {
  t: number[];
  d: number[];
  speed: number[];
  throttle: number[];
  brake: number[];
  steering: number[];
  clutch: number[];
  gear: number[];
  rpm: number[];
  x: number[];
  z: number[];
  gLat: number[];
  gLong: number[];
  abs: number[];
  tc: number[];
  tyreFl: number[];
  tyreFr: number[];
  tyreRl: number[];
  tyreRr: number[];
}

export interface TrackMap {
  track: string;
  length: number;
  points: [number, number][];
  dist: number[];
}

export interface PersonalBest {
  track: string;
  car: string;
  carClass: string;
  lapId: number;
  lapTime: number;
  s1: number | null;
  s2: number | null;
  s3: number | null;
  recordedAt: number;
  lapCount: number;
  /** Recorded in the game, not demo data; only these can go online. */
  fromGame: boolean;
}

export interface Filters {
  tracks: string[];
  cars: string[];
}

export interface LapRecordedEvent {
  lapId: number;
  sessionId: number;
  lapNumber: number;
  lapTime: number;
  valid: boolean;
  personalBest: boolean;
  track: string;
}

export interface DailyRace {
  name: string;
  tier: string;
  track: string;
  duration: string;
  image: string | null;
  /** Start times as Unix milliseconds. */
  starts: number[];
}

/** This install's account on the Mulsanne server. */
export interface OnlineMe {
  id: string;
  /** `null` shows as "Anonymous". */
  displayName: string | null;
  /** Unix seconds. */
  createdAt: number;
}

/** queued → ranked | held, or failed (retried) | rejected (by the server). */
export type UploadStatus = "queued" | "ranked" | "held" | "failed" | "rejected";

export interface UploadState {
  lapId: number;
  status: UploadStatus;
  remoteId: string | null;
  error: string | null;
  /** Telemetry: null (not sent), queued, uploaded, replaced (by a faster lap's), skipped or rejected. */
  trace: "queued" | "uploaded" | "replaced" | "skipped" | "rejected" | null;
}

export interface LeaderboardEntry {
  rank: number;
  lapId: string;
  /** `null` for anonymous laps. */
  driver: string | null;
  car: string;
  vehicleName: string;
  sessionType: string;
  /** LMU build number, when known. */
  gameVersion: number | null;
  /** Its telemetry can be downloaded for comparison. */
  hasTrace: boolean;
  lapTime: number;
  s1: number;
  s2: number;
  s3: number;
  /** Unix seconds. */
  recordedAt: number;
}

export interface Leaderboard {
  entries: LeaderboardEntry[];
  total: number;
}

export interface BoardSummary {
  track: string;
  carClass: string;
  drivers: number;
  bestLap: number;
  /** Game versions with ranked laps, newest first. */
  gameVersions: number[];
}
