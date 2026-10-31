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

export interface ChangelogRelease {
  version: string;
  date: string;
  sections: ChangelogSection[];
}

function compareVersions(a: string, b: string): number {
  const pa = a.split(/[.-]/).map(Number);
  const pb = b.split(/[.-]/).map(Number);
  for (let i = 0; i < 3; i++) {
    const d = (pa[i] || 0) - (pb[i] || 0);
    if (d) return d;
  }
  return 0;
}

/**
 * Read releases from release-please's generated changelog, newest first: the installed
 * `version` plus every older release newer than `since` (the version the user last saw).
 * Without `since`, or when it isn't older than `version`, only the installed release is returned.
 */
export function parseChangelog(markdown: string, version: string, since = ""): ChangelogRelease[] {
  const current = version.replace(/^v/, "");
  const previous = since.replace(/^v/, "");
  const includeOlder = previous !== "" && compareVersions(previous, current) < 0;
  const releases: ChangelogRelease[] = [];
  let release: ChangelogRelease | undefined;
  let section: ChangelogSection | undefined;
  let lines: string[] | undefined;
  const raw: { section: ChangelogSection; lines: string[] }[] = [];

  for (const line of markdown.split(/\r?\n/)) {
    const header = line.match(/^##\s+\[?v?(\d+\.\d+\.\d+(?:-[\w.-]+)?)(?=[\]\s(]|$)(?:.*\((\d{4}-\d{2}-\d{2})\))?/);
    if (header) {
      const v = header[1];
      section = undefined;
      lines = undefined;
      if (release && (!includeOlder || compareVersions(v, previous) <= 0)) break;
      release = release || v === current ? { version: v, date: header[2] ?? "", sections: [] } : undefined;
      if (release) releases.push(release);
      continue;
    }
    if (!release) continue;

    const heading = line.match(/^###\s+(.+)/);
    if (heading) {
      section = { title: plainText(heading[1]), entries: [] };
      release.sections.push(section);
      lines = undefined;
      continue;
    }

    const bullet = line.match(/^\s*[-*]\s+(.+)/);
    if (bullet) {
      if (!section) {
        section = { title: "Changes", entries: [] };
        release.sections.push(section);
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

  return releases
    .map((r) => ({ ...r, sections: r.sections.filter((s) => s.entries.length > 0) }))
    .filter((r) => r.sections.length > 0);
}
