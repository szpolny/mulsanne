export type Page = "live" | "overlays" | "sessions" | "analysis" | "records" | "settings";

class Nav {
  page = $state<Page>("live");
  /** Session opened in the Sessions page. */
  sessionId = $state<number | null>(null);
  /** Laps loaded in the Analysis page. */
  lapA = $state<number | null>(null);
  lapB = $state<number | null>(null);
  track = $state<string | null>(null);

  openAnalysis(track: string, lapA: number, lapB: number | null = null) {
    this.track = track;
    this.lapA = lapA;
    this.lapB = lapB;
    this.page = "analysis";
  }
}

export const nav = new Nav();
