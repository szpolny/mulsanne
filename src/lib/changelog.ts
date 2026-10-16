export interface ChangelogSection {
  title: string;
  entries: string[];
}

function plainText(text: string): string {
  return text
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/\*\*|__|`/g, "")
    .trim();
}

/** Read one installed release from release-please's generated changelog. */
export function parseChangelog(markdown: string, version: string): ChangelogSection[] {
  const sections: ChangelogSection[] = [];
  let inRelease = false;
  let section: ChangelogSection | undefined;

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
      continue;
    }

    const bullet = line.match(/^\s*[-*]\s+(.+)/);
    if (bullet) {
      // Commit links are useful on GitHub, but add noise to the in-app notes.
      const entry = plainText(bullet[1].replace(/\s*\(\[[\da-f]{7,40}\]\([^)]+\)\)\s*$/i, ""));
      if (!entry) continue;
      if (!section) {
        section = { title: "Changes", entries: [] };
        sections.push(section);
      }
      section.entries.push(entry.charAt(0).toUpperCase() + entry.slice(1));
    } else if (/^\s+\S/.test(line) && section?.entries.length) {
      section.entries[section.entries.length - 1] += ` ${plainText(line)}`;
    }
  }

  return sections.filter((section) => section.entries.length > 0);
}
