import { OUTLINES, type TrackOutline } from "./outlines";

// Variants before their base track.
const MATCHERS: [RegExp, string][] = [
  [/sarthe|le ?mans/, "le-mans"],
  [/spa/, "spa"],
  [/monza/, "monza"],
  [/fuji/, "fuji"],
  [/bahrain.*outer/, "bahrain-outer"],
  [/bahrain.*endurance/, "bahrain-endurance"],
  [/bahrain.*paddock/, "bahrain-paddock"],
  [/bahrain|sakhir/, "bahrain"],
  [/portim|algarve/, "portimao"],
  [/sebring/, "sebring"],
  [/imola|enzo e dino/, "imola"],
  [/interlagos|sao paulo|carlos pace/, "interlagos"],
  [/lusail|losail|qatar/, "lusail"],
  [/(americas|cota).*national/, "cota-national"],
  [/americas|cota/, "cota"],
  [/ricard|castellet/, "paul-ricard"],
  [/silverstone/, "silverstone"],
  [/barcelona|catalunya/, "barcelona"],
  [/laguna/, "laguna-seca"],
  [/long ?beach/, "long-beach"],
];

/** Finds the outline for a track name as LMU or RaceControl writes it. */
export function trackOutline(name: string): TrackOutline | null {
  const n = name
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "");
  for (const [re, id] of MATCHERS) if (re.test(n)) return OUTLINES[id] ?? null;
  return null;
}
