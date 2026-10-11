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
    private syncing = false;

    zoom(from: uPlot, min: number, max: number) {
      if (this.syncing) return;
      this.syncing = true;
      for (const u of this.charts) if (u !== from) u.setScale("x", { min, max });
      this.syncing = false;
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
      },
    };
    const data = [x, ...series.map((s) => s.data)] as uPlot.AlignedData;
    chart = new uPlot(opts, data, el);
    group.charts.add(chart);
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
  <div bind:clientWidth={width}>
    <div bind:this={el}></div>
  </div>
</div>
