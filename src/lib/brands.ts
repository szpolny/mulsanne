import {
  siAstonmartin,
  siAudi,
  siBmw,
  siCadillac,
  siChevrolet,
  siFerrari,
  siFord,
  siHonda,
  siLamborghini,
  siMclaren,
  siNissan,
  siPeugeot,
  siPorsche,
  siToyota,
} from "simple-icons";
import { LOGOS, type LogoData } from "./logos/logos";
// Raster logos converted to single-colour alpha masks by scripts/make-logo-masks.ps1.
import alpineMask from "./logos/masks/alpine.png";
import duqueineMask from "./logos/masks/duqueine.png";
import ginettaMask from "./logos/masks/ginetta.png";
import ligierMask from "./logos/masks/ligier.png";

export interface Brand {
  name: string;
  /** Vector logo, when one is available. */
  logo?: LogoData;
  /** URL of a white-on-transparent mask image, tinted with the text colour. */
  mask?: string;
  /** Short monogram used when there is no logo. */
  mono: string;
}

/** Simple Icons glyphs use a 24×24 box. */
const si = (icon: { path: string }): LogoData => ({ viewBox: "0 0 24 24", paths: [{ d: icon.path }] });

/** Just the speedmark (the first subpath) of the Simple Icons McLaren logo, without the wordmark. */
const mclarenSpeedmark: LogoData = {
  viewBox: "19.06 10.21 4.94 2.75",
  paths: [{ d: siMclaren.path.slice(0, siMclaren.path.indexOf("z") + 1) }],
};

const BRANDS: { match: RegExp; brand: Brand }[] = [
  { match: /porsche/i, brand: { name: "Porsche", logo: si(siPorsche), mono: "P" } },
  { match: /ferrari/i, brand: { name: "Ferrari", logo: si(siFerrari), mono: "F" } },
  { match: /bmw/i, brand: { name: "BMW", logo: si(siBmw), mono: "BMW" } },
  { match: /toyota|gr010|tr010/i, brand: { name: "Toyota", logo: si(siToyota), mono: "T" } },
  { match: /cadillac/i, brand: { name: "Cadillac", logo: si(siCadillac), mono: "C" } },
  { match: /peugeot|9x8/i, brand: { name: "Peugeot", logo: si(siPeugeot), mono: "P" } },
  { match: /aston/i, brand: { name: "Aston Martin", logo: si(siAstonmartin), mono: "AM" } },
  { match: /lamborghini|huracan|sc63/i, brand: { name: "Lamborghini", logo: si(siLamborghini), mono: "L" } },
  { match: /corvette|chevrolet/i, brand: { name: "Chevrolet", logo: si(siChevrolet), mono: "C" } },
  { match: /mustang|ford/i, brand: { name: "Ford", logo: si(siFord), mono: "F" } },
  { match: /mclaren/i, brand: { name: "McLaren", logo: mclarenSpeedmark, mono: "M" } },
  { match: /audi/i, brand: { name: "Audi", logo: si(siAudi), mono: "A" } },
  { match: /honda|acura|nsx/i, brand: { name: "Honda", logo: si(siHonda), mono: "H" } },
  { match: /nissan/i, brand: { name: "Nissan", logo: si(siNissan), mono: "N" } },
  { match: /mercedes|amg/i, brand: { name: "Mercedes-AMG", logo: LOGOS.mercedes, mono: "MB" } },
  { match: /lexus|rc ?f/i, brand: { name: "Lexus", logo: LOGOS.lexus, mono: "LX" } },
  { match: /genesis|gmr/i, brand: { name: "Genesis", mono: "G" } },
  { match: /alpine|a424/i, brand: { name: "Alpine", mask: alpineMask, mono: "A" } },
  { match: /oreca/i, brand: { name: "Oreca", logo: LOGOS.oreca, mono: "OR" } },
  { match: /ligier|jsp/i, brand: { name: "Ligier", mask: ligierMask, mono: "LG" } },
  { match: /duqueine/i, brand: { name: "Duqueine", mask: duqueineMask, mono: "DQ" } },
  { match: /ginetta/i, brand: { name: "Ginetta", mask: ginettaMask, mono: "GI" } },
  { match: /adess/i, brand: { name: "ADESS", mono: "AD" } },
  { match: /glickenhaus|scg/i, brand: { name: "Glickenhaus", mono: "SCG" } },
  { match: /vanwall/i, brand: { name: "Vanwall", mono: "V" } },
  { match: /isotta/i, brand: { name: "Isotta Fraschini", logo: LOGOS.isotta, mono: "IF" } },
];

const cache = new Map<string, Brand | null>();

/** Finds the manufacturer from the car model, falling back to the entry name. */
export function brandOf(model: string, entry = ""): Brand | null {
  const key = `${model}|${entry}`;
  if (cache.has(key)) return cache.get(key)!;
  const hit = BRANDS.find((b) => b.match.test(model)) ?? BRANDS.find((b) => b.match.test(entry)) ?? null;
  const brand = hit?.brand ?? null;
  cache.set(key, brand);
  return brand;
}

/** Tyre compound colours, Pirelli/Michelin style. */
export function compoundStyle(name: string): { letter: string; color: string } {
  const n = name.toLowerCase();
  if (n.startsWith("s")) return { letter: "S", color: "#ef3b3b" };
  if (n.startsWith("m")) return { letter: "M", color: "#f5c518" };
  if (n.startsWith("h")) return { letter: "H", color: "#e8e8ec" };
  if (n.startsWith("w") || n.includes("rain")) return { letter: "W", color: "#3b8cff" };
  if (n.startsWith("i")) return { letter: "I", color: "#2fbf5b" };
  return { letter: name ? name[0].toUpperCase() : "?", color: "#8f8f9b" };
}
