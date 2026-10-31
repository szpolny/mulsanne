export interface ChangelogRef {
  label: string;
  url: string;
}

export interface ChangelogEntry {
  text: string;
  /** Pull requests that introduced the change. */
  refs: ChangelogRef[];
}

export interface ChangelogSection {
  title: string;
  entries: ChangelogEntry[];
}

function plainText(text: string): string {
  return text
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/\*\*|__|`/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

/**
 * release-please appends `([#11](url))`, `([abc1234](url))` and `, closes [#9](url)` to entries.
 * Those read badly as plain text, so strip them and keep PR links as separate refs.
 */
function parseEntry(raw: string): ChangelogEntry {
  const refs: ChangelogRef[] = [];
  const text = raw
    // ", closes #8 #10" — the linked issues are reachable from the PR.
    .replace(/,?\s*closes\s+(?:\[#\d+\]\([^)]+\)|#\d+)(?:\s*,?\s+(?:\[#\d+\]\([^)]+\)|#\d+))*/gi, "")
    // Commit links.
    .replace(/\s*\(\[[\da-f]{7,40}\]\([^)]+\)\)/gi, "")
    // PR links.
    .replace(/\s*\(\[(#\d+)\]\(([^)]+)\)\)/g, (_, label: string, url: string) => {
      refs.push({ label, url });
      return "";
    });

  const clean = plainText(text);
  return { text: clean.charAt(0).toUpperCase() + clean.slice(1), refs };
}

/** Read one installed release from release-please's generated changelog. */
export function parseChangelog(markdown: string, version: string): ChangelogSection[] {
  const sections: ChangelogSection[] = [];
  let inRelease = false;
  let section: ChangelogSection | undefined;
  let lines: string[] | undefined;
  const raw: { section: ChangelogSection; lines: string[] }[] = [];

  for (const line of markdown.split(/\r?\n/)) {
    const release = line.match(/^##\s+\[?v?(\d+\.\d+\.\d+(?:-[\w.-]+)?)(?=[\]\s(]|$)/);
    if (release) {
      if (inRelease) break;
      inRelease = release[1] === version.replace(/^v/, "");
      continue;
    }
    if (!inRelease) continue;

    const heading = line.match(/^###\s+(.+)/);
    if (heading) {
      section = { title: plainText(heading[1]), entries: [] };
      sections.push(section);
      lines = undefined;
      continue;
    }

    const bullet = line.match(/^\s*[-*]\s+(.+)/);
    if (bullet) {
      if (!section) {
        section = { title: "Changes", entries: [] };
        sections.push(section);
      }
      lines = [bullet[1]];
      raw.push({ section, lines });
    } else if (/^\s+\S/.test(line) && lines) {
      // Wrapped continuation of the previous bullet.
      lines.push(line.trim());
    }
  }

  for (const { section, lines } of raw) {
    const entry = parseEntry(lines.join(" "));
    if (entry.text) section.entries.push(entry);
  }

  return sections.filter((section) => section.entries.length > 0);
}
