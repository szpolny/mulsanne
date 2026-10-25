<script lang="ts" module>
  import uPlot from "uplot";

  export interface Series {
    label: string;
    data: ArrayLike<number>;
    color: string;
    width?: number;
    dash?: number[];
    stepped?: boolean;
    fill?: string;
  }

  /** All charts on the page share cursor and x-zoom through this group. */
  export class ChartGroup {
    charts = new Set<uPlot>();
    /** Full x extent of the data and the zoomed window, or null when showing everything. */
    extent = $state<[number, number]>([0, 0]);
    range = $state<[number, number] | null>(null);
    private syncing = false;

    zoom(from: uPlot, min: number, max: number) {
      if (this.syncing) return;
      const xs = from.data[0];
      const lo = xs[0];
      const hi = xs[xs.length - 1];
      this.extent = [lo, hi];
      this.range = min <= lo && max >= hi ? null : [min, max];
      this.syncing = true;
      for (const u of this.charts) if (u !== from) u.setScale("x", { min, max });
      this.syncing = false;
    }

    /** Zoom to [min, max], clamped to the data and at least `minSpan` wide. */
    zoomTo(min: number, max: number, minSpan = 30) {
      const u = this.charts.values().next().value;
      if (!u) return;
      const [lo, hi] = [u.data[0][0], u.data[0][u.data[0].length - 1]];
      const span = Math.min(hi - lo, Math.max(minSpan, max - min));
      min = Math.max(lo, Math.min(hi - span, min));
      u.setScale("x", { min, max: min + span });
    }

    /** Scale the visible span by `f` (< 1 zooms in) around `at`, the middle by default. */
    zoomBy(f: number, at?: number) {
      const [min, max] = this.range ?? this.extent;
      const c = at ?? (min + max) / 2;
      this.zoomTo(c - (c - min) * f, c + (max - c) * f);
    }

    reset() {
      this.zoomTo(-Infinity, Infinity);
    }
  }
</script>

<script lang="ts">
  import "uplot/dist/uPlot.min.css";
  import { onDestroy, untrack } from "svelte";

  let {
    title,
    x,
    series,
    height = 140,
    yRange,
    group,
    syncKey,
    cursorIdx,
    oncursor,
    format = (v: number) => v.toFixed(0),
  }: {
    title: string;
    x: ArrayLike<number>;
    series: Series[];
    height?: number;
    yRange?: [number, number];
    group: ChartGroup;
    syncKey: string;
    cursorIdx: number | null;
    oncursor: (idx: number | null) => void;
    format?: (v: number) => string;
  } = $props();

  let el: HTMLDivElement;
  let chart: uPlot | null = null;
  let width = $state(600);
  let lastIdx: number | null = null;

  const AXIS = { stroke: "#8f8f9b", grid: { stroke: "#1f1f25", width: 1 }, ticks: { stroke: "#1f1f25", width: 1 }, font: "500 12px Barlow Condensed" };

  function build() {
    chart?.destroy();
    if (chart) group.charts.delete(chart);
    const opts: uPlot.Options = {
      width,
      height,
      legend: { show: false },
      padding: [8, 8, 0, 0],
      cursor: {
        sync: { key: syncKey, setSeries: false },
        points: { size: 6 },
        drag: { x: true, y: false, setScale: true },
      },
      scales: { x: { time: false }, y: yRange ? { range: yRange } : { auto: true } },
      axes: [
        { ...AXIS, values: (_, ticks) => ticks.map((t) => `${Math.round(t)} m`), size: 28 },
        { ...AXIS, size: 44, values: (_, ticks) => ticks.map(format) },
      ],
      series: [
        {},
        ...series.map((s) => ({
          label: s.label,
          stroke: s.color,
          width: s.width ?? 1.5,
          dash: s.dash,
          fill: s.fill,
          paths: s.stepped ? uPlot.paths.stepped!({ align: 1 }) : undefined,
          points: { show: false },
        })),
      ],
      hooks: {
        setCursor: [
          (u) => {
            const idx = u.cursor.idx ?? null;
            if (idx !== lastIdx) {
              lastIdx = idx;
              oncursor(idx);
            }
          },
        ],
        setScale: [
          (u, key) => {
            if (key === "x") group.zoom(u, u.scales.x.min!, u.scales.x.max!);
          },
        ],
        setSize: [measure],
        ready: [measure],
      },
    };
    const data = [x, ...series.map((s) => s.data)] as uPlot.AlignedData;
    chart = new uPlot(opts, data, el);
    group.charts.add(chart);
    const u = chart;
    // Plain scroll keeps scrolling the page (with a hint on how to zoom); Ctrl zooms around the cursor, Shift pans.
    u.over.addEventListener(
      "wheel",
      (e) => {
        if (!e.ctrlKey && !e.shiftKey) {
          if (hovered) flashHint();
          return;
        }
        e.preventDefault();
        if (e.ctrlKey) group.zoomBy(e.deltaY < 0 ? 0.8 : 1.25, u.posToVal(e.clientX - u.over.getBoundingClientRect().left, "x"));
        else panBy(Math.sign(e.deltaX || e.deltaY) * 0.15);
      },
      { passive: false },
    );
    // Only hint after the pointer moved here, not when the page scrolls a chart under a still mouse.
    u.over.addEventListener("pointermove", () => (hovered = true));
    u.over.addEventListener("pointerleave", () => (hovered = false));
  }

  /** Plot area in CSS pixels, relative to the chart element. */
  let plot = $state({ left: 0, top: 0, width: 0, height: 0 });
  function measure(u: uPlot) {
    const r = uPlot.pxRatio;
    plot = { left: u.bbox.left / r, top: u.bbox.top / r, width: u.bbox.width / r, height: u.bbox.height / r };
  }

  let hovered = false;
  let hint = $state(false);
  let hintTimer: ReturnType<typeof setTimeout>;
  function flashHint() {
    hint = true;
    clearTimeout(hintTimer);
    hintTimer = setTimeout(() => (hint = false), 1200);
  }

  /** Move the zoomed window by a fraction of its width. */
  function panBy(frac: number) {
    if (!group.range) return;
    const [min, max] = group.range;
    const d = (max - min) * frac;
    group.zoomTo(min + d, max + d);
  }

  // Scrollbar along the bottom of the plot while zoomed: drag the thumb to pan, click the track to jump.
  const thumb = $derived.by(() => {
    if (!group.range) return null;
    const [lo, hi] = group.extent;
    const [min, max] = group.range;
    return { left: ((min - lo) / (hi - lo)) * 100, width: Math.max(2, ((max - min) / (hi - lo)) * 100) };
  });
  let pan: { x: number; min: number; max: number } | null = null;
  function onThumbDown(e: PointerEvent) {
    if (!group.range) return;
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    pan = { x: e.clientX, min: group.range[0], max: group.range[1] };
  }
  function onThumbMove(e: PointerEvent) {
    if (!pan) return;
    const [lo, hi] = group.extent;
    const d = ((e.clientX - pan.x) / plot.width) * (hi - lo);
    group.zoomTo(pan.min + d, pan.max + d);
  }
  function onTrackDown(e: PointerEvent) {
    if (!group.range) return;
    const [lo, hi] = group.extent;
    const [min, max] = group.range;
    const at = lo + ((e.clientX - (e.currentTarget as HTMLElement).getBoundingClientRect().left) / plot.width) * (hi - lo);
    group.zoomTo(at - (max - min) / 2, at + (max - min) / 2);
  }

  // Rebuild only when data or series change; width changes just resize.
  $effect(() => {
    void x;
    void series;
    void height;
    untrack(build);
  });
  $effect(() => {
    chart?.setSize({ width, height });
  });

  // Follow cursor updates coming from the map.
  $effect(() => {
    const idx = cursorIdx;
    if (!chart || idx == null || idx === lastIdx) return;
    lastIdx = idx;
    const left = chart.valToPos(x[idx], "x");
    chart.setCursor({ left, top: chart.cursor.top && chart.cursor.top > 0 ? chart.cursor.top : height / 2 }, false);
  });

  onDestroy(() => {
    clearTimeout(hintTimer);
    if (chart) group.charts.delete(chart);
    chart?.destroy();
  });

  const legendValues = $derived(
    cursorIdx == null ? series.map(() => "") : series.map((s) => format(s.data[cursorIdx] ?? NaN)),
  );
</script>

<div class="card px-2 pt-1.5 pb-1">
  <div class="mb-0.5 flex items-center gap-4 px-1 text-xs">
    <span class="font-semibold text-muted">{title}</span>
    {#each series as s, i}
      <span class="flex items-center gap-1.5 tnum">
        <span class="inline-block h-0.5 w-3" style:background={s.color}></span>
        <span class="text-muted">{s.label}</span>
        <span>{legendValues[i]}</span>
      </span>
    {/each}
  </div>
  <div class="group relative" bind:clientWidth={width}>
    <div bind:this={el}></div>

    <div
      class={[
        "absolute top-1 flex items-center gap-0.5 rounded-md border border-line bg-surface-2 p-0.5 shadow-md text-xs transition-opacity",
        "opacity-0 group-hover:opacity-100 focus-within:opacity-100",
      ]}
      style:right="{width - plot.left - plot.width + 4}px"
    >
      {#if group.range}
        <span class="px-1.5 tnum text-muted">{Math.round(group.range[0])}–{Math.round(group.range[1])} m</span>
      {:else}
        <span class="px-1.5 text-faint">Drag or <span class="kbd">Ctrl</span> + scroll to zoom</span>
      {/if}
      <button class="zoom-btn" title="Zoom out" aria-label="Zoom out" disabled={!group.range} onclick={() => group.zoomBy(2)}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true"><path d="M3 7h8" stroke="currentColor" stroke-width="1.4" /></svg>
      </button>
      <button class="zoom-btn" title="Zoom in" aria-label="Zoom in" onclick={() => group.zoomBy(0.5)}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true"><path d="M3 7h8M7 3v8" stroke="currentColor" stroke-width="1.4" /></svg>
      </button>
      {#if group.range}
        <button class="zoom-btn px-1.5" title="Show the whole lap (Esc or double-click)" onclick={() => group.reset()}>Reset</button>
      {/if}
    </div>

    {#if thumb}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="absolute h-1.5 cursor-pointer rounded-full bg-surface-3/80"
        style:left="{plot.left}px"
        style:width="{plot.width}px"
        style:top="{plot.top + plot.height - 8}px"
        onpointerdown={onTrackDown}
      >
        <div
          class="absolute inset-y-0 cursor-grab rounded-full bg-accent-hi active:cursor-grabbing"
          style:left="{thumb.left}%"
          style:width="{thumb.width}%"
          title="Drag to pan"
          onpointerdown={onThumbDown}
          onpointermove={onThumbMove}
          onpointerup={() => (pan = null)}
        ></div>
      </div>
    {/if}

    {#if hint}
      <div
        class="pointer-events-none absolute flex items-center justify-center bg-bg/60 text-sm text-fg"
        style:left="{plot.left}px"
        style:top="{plot.top}px"
        style:width="{plot.width}px"
        style:height="{plot.height}px"
      >
        Hold <span class="kbd mx-1">Ctrl</span> while scrolling to zoom
      </div>
    {/if}
  </div>
</div>

<style>
  .zoom-btn {
    display: flex;
    height: 1.5rem;
    min-width: 1.5rem;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    color: var(--color-muted);
  }
  .zoom-btn:hover:not(:disabled) {
    background: var(--color-surface-3);
    color: var(--color-fg);
  }
  .zoom-btn:disabled {
    opacity: 0.35;
  }
</style>
