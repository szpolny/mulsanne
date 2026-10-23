export const PRIVACY_URL = "https://mulsanne.xyz/privacy#online-features";

// Racing words for generated names, Reddit-style: Adjective_Noun_1234.
const ADJECTIVES = [
  "Swift", "Late", "Early", "Flying", "Silent", "Rapid", "Bold", "Steady", "Smooth", "Brave",
  "Clever", "Lucky", "Fearless", "Patient", "Restless", "Daring", "Crafty", "Nimble", "Mighty", "Gentle",
  "Wild", "Calm", "Keen", "Quiet", "Sharp", "Sunny", "Rainy", "Misty", "Midnight", "Golden",
];
const NOUNS = [
  "Apex", "Hairpin", "Chicane", "Kerb", "Slipstream", "Pitlane", "Gearbox", "Diffuser", "Splitter", "Wing",
  "Brake", "Throttle", "Piston", "Turbo", "Tyre", "Stint", "Sector", "Paddock", "Grid", "Flag",
  "Marshal", "Spanner", "Torque", "Camber", "Downforce", "Overtake", "Podium", "Hypercar", "Prototype", "Hybrid",
];

function pick<T>(items: T[], random: number): T {
  return items[random % items.length];
}

/** A random display name like `Late_Apex_4821`; not guaranteed unique. */
export function generateName(): string {
  const [a, b, c] = crypto.getRandomValues(new Uint32Array(3));
  return `${pick(ADJECTIVES, a)}_${pick(NOUNS, b)}_${1000 + (c % 9000)}`;
}

/** Mirrors the server's display name rules; empty means anonymous. */
export function displayNameError(name: string): string | null {
  const trimmed = name.trim();
  if (!trimmed) return null;
  const length = [...trimmed].length;
  if (length < 2 || length > 32) return "Display name must be 2 to 32 characters.";
  if (/\p{Cc}/u.test(trimmed)) return "Display name contains invalid characters.";
  return null;
}
