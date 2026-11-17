<script lang="ts">
  /** Small line icon for an overlay widget, drawn on a 24×24 grid. */
  let { id, class: klass = "" }: { id: string; class?: string } = $props();

  const PATHS: Record<string, string> = {
    dashboard: "M4 16a8 8 0 1 1 16 0M12 16l4-5M8 20h8",
    inputs: "M5 20V10M10 20V4M15 20v-7M20 20V8",
    delta: "M4 12h16M12 4v16M7 7l10 10",
    laptiming: "M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM12 13V9M10 2h4M18.5 5.5l1.5-1.5",
    standings: "M4 6h2M4 12h2M4 18h2M10 6h10M10 12h10M10 18h10",
    relative: "M8 20V4M4 8l4-4 4 4M16 4v16M12 16l4 4 4-4",
    fuel: "M5 21V5a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v16M3 21h14M5 11h10M15 8h2a2 2 0 0 1 2 2v6a2 2 0 0 0 4 0V9l-3-3",
    tyres: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM12 3v6M12 15v6M3 12h6M15 12h6",
    session: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",
    flags: "M5 21V4M5 4h12l-2 4 2 4H5",
    trackmap: "M6 19c-2 0-3-2-2-4l3-6c1-2 3-2 4 0l1 2c1 2 3 2 4 0l1-2c1-2 4-1 4 1v6c0 2-1 3-3 3z",
    weather: "M7 18h10a4 4 0 0 0 .5-8A6 6 0 0 0 6 11a3.5 3.5 0 0 0 1 7z",
    damage: "M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.4-.6-.6-2.4z",
    electronics: "M13 2 5 14h6l-1 8 8-12h-6z",
  };
</script>

<svg
  viewBox="0 0 24 24"
  class={klass}
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={PATHS[id] ?? "M4 4h16v16H4z"} />
</svg>
