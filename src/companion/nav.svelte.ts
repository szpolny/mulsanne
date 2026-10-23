/** A lap from the online leaderboards, opened for comparison. */
export interface RankedLap {
  /** Server lap id. */
  id: string;
  /** Driver (or "Anonymous") and car. */
  label: string;
  lapTime: number;
  s1: number;
  s2: number;
  s3: number;
}

export type Page = "live" | "overlays" | "sessions" | "analysis" | "records" | "rankings" | "schedule" | "settings";

class Nav {
  page = $state<Page>("live");
  /** Session opened in the Sessions page. */
  sessionId = $state<number | null>(null);
  /** Laps loaded in the Analysis page. */
  lapA = $state<number | null>(null);
  lapB = $state<number | null>(null);
  /** A ranked lap from the online leaderboards to compare with, instead of `lapB`. */
  remoteB = $state<RankedLap | null>(null);
  track = $state<string | null>(null);

  openAnalysis(track: string, lapA: number, lapB: number | null = null) {
    this.track = track;
    this.lapA = lapA;
    this.lapB = lapB;
    this.remoteB = null;
    this.page = "analysis";
  }

  /** Compares your best lap on `track` with a ranked lap. */
  compareWithRanked(track: string, remote: RankedLap) {
    if (this.track !== track) this.lapA = null;
    this.track = track;
    this.lapB = null;
    this.remoteB = remote;
    this.page = "analysis";
  }
}

export const nav = new Nav();
