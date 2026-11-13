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

/** The lap's own livery first, then plain ones of the same model. */
export function liveryCandidates(vehFile: string | null | undefined, car: string, carClass = "", vehicleName = ""): string[] {
  return [...(vehFile ? [vehFile] : []), ...plainLiveries(car, carClass, vehicleName)];
}
