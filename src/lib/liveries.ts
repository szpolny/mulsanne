// LMU ships a plain "custom livery" render of every model as car #397,
// e.g. `397_26_499P` or `397_26ELMS_ORECA07`. They stand in for laps whose
// livery is unknown, or isn't in this player's install.

// Most specific first: several models share a maker.
const MODELS: [RegExp, string][] = [
  // Hypercar
  [/499p/, "499P"],
  [/963/, "963"],
  [/9x8/, "9X8W"],
  [/gr010/, "GR010"],
  [/cadillac|v-series/, "VLMDH"],
  [/m hybrid|bmw.*v8/, "BMWMH"],
  [/alpine|a424/, "ALPINE"],
  [/sc63/, "SC63"],
  [/isotta|tipo ?6/, "ISOTTA"],
  [/glickenhaus|scg ?007/, "GLICK"],
  [/vanwall/, "VANWALL"],
  [/valkyrie/, "AMVALK"],
  [/genesis|gmr/, "GMR001"],
  // LMP2 / LMP3
  [/oreca/, "ORECA07"],
  [/ligier|js ?p3/, "JSP325"],
  [/duqueine|d09/, "D09P3"],
  [/ginetta|g61/, "G61LTP3EVO"],
  [/adess|ad25/, "AD25"],
  // GTE (2023)
  [/488/, "488GTE"],
  [/911 rsr|911.*gte/, "911GTE"],
  [/c8\.?r\b|corvette.*gte/, "C8RGTE"],
  // LMGT3
  [/296/, "296GT3"],
  [/911/, "911GT3R"],
  [/z06|corvette/, "Z06GT3R"],
  [/mclaren|720s/, "MCLAREN"],
  [/mustang/, "MUSTANG"],
  [/lexus|rc ?f/, "LEXUS"],
  [/huracan|lamborghini/, "HURACAN"],
  [/bmw|m4/, "BMW"],
  [/mercedes|amg/, "AMG"],
  [/aston|vantage/, "AMV"],
];

/** Newest season first; the game only has some models in some seasons. */
const SEASONS = ["26", "25", "24", "23"];

/** The game's model code for a car name as LMU reports it, e.g. "Ferrari 499P" -> "499P". */
export function modelCode(car: string): string | null {
  const c = car
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "");
  for (const [re, code] of MODELS) if (re.test(c)) return code;
  return null;
}

/**
 * Plain liveries to try for a car, best first. ELMS-spec cars (by class, or
 * the ":ELMS" tag LMU puts on their vehicle names) prefer the ELMS render.
 */
export function plainLiveries(car: string, carClass = "", vehicleName = ""): string[] {
  const code = modelCode(car);
  if (!code) return [];
  const elms = /elms/i.test(carClass) || /:elms\b/i.test(vehicleName);
  const seasons = elms ? [...SEASONS.map((s) => `${s}ELMS`), ...SEASONS] : SEASONS;
  return seasons.map((s) => `397_${s}_${code}`);
}

/**
 * The lap's own livery first, then the one its vehicle name points to (laps
 * uploaded before liveries were sent have only the name), then plain ones of
 * the same model. `names` are the renders the game has.
 */
export function liveryCandidates(
  vehFile: string | null | undefined,
  car: string,
  carClass = "",
  vehicleName = "",
  names: readonly string[] = [],
): string[] {
  const named = liveryForVehicle(vehicleName, names);
  return [...new Set([...(vehFile ? [vehFile] : []), ...(named ? [named] : []), ...plainLiveries(car, carClass, vehicleName)])];
}

/**
 * The game's render of a livery from its vehicle name, for laps uploaded
 * without the livery file. `names` are the renders the game has, e.g.
 * `36_26_ALPI20490502` for "Alpine Endurance Team 2026 #36:WEC": the car
 * number, the season and a team code, then a hash. 2023 liveries carry no
 * season (`83_AFCORSE7FBCECA5`).
 */
export function liveryForVehicle(vehicleName: string | null | undefined, names: readonly string[]): string | null {
  const m = /^(.+?)\s+(?:20(\d\d)\s+)?#(\d+)\b/.exec(vehicleName ?? "");
  if (!m) return null;
  const [, team, season, number] = m;
  const words = team
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .split(/[^a-z0-9]+/)
    .filter(Boolean);
  const pick = (prefix: string, rest: RegExp) =>
    names.find((n) => n.startsWith(prefix) && rest.test(n.slice(prefix.length)) && teamMatches(teamCode(n.slice(prefix.length)), words)) ?? null;
  return (
    (season ? pick(`${number}_${season}_`, /^[A-Z]/) : null) ??
    (!season || season === "23" ? pick(`${number}_`, /^[A-Z]/) : null)
  );
}

/** "JOTAA5525C5E" -> "jota", "UNITED_18140FB8" -> "united". */
function teamCode(rest: string): string {
  const code = /^(.*?)_?[0-9A-F]{8}$/.exec(rest)?.[1] || /^[A-Z]+/.exec(rest)?.[0] || "";
  return code.replace(/_+$/, "").toLowerCase();
}

/** Whether a team code abbreviates the team: "rsl" for Racing Spirit of Léman, "thor" for (The) Heart of Racing. */
function teamMatches(code: string, words: readonly string[]): boolean {
  if (!code) return false;
  return abbreviates(code, words, 0) || (code.length > 3 && code.startsWith("t") && abbreviates(code.slice(1), words, 0));
}

/** Each piece of `code` is the start of a word, in order; words may be skipped. */
function abbreviates(code: string, words: readonly string[], from: number): boolean {
  if (!code) return true;
  for (let i = from; i < words.length; i++) {
    for (let n = Math.min(code.length, words[i].length); n > 0; n--) {
      if (words[i].startsWith(code.slice(0, n)) && abbreviates(code.slice(n), words, i + 1)) return true;
    }
  }
  return false;
}
