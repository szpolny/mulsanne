<script lang="ts">
  import { onMount } from "svelte";
  import { align, brakeZones, type Aligned, type BrakeZone } from "../../lib/analysis";
  import { api } from "../../lib/api";
  import { config } from "../../lib/config.svelte";
  import { dateTime, delta as fmtDelta, lapTime, sectorTime, speed as toSpeed, speedUnit, temp } from "../../lib/format";
  import type { LapRecord, LapTrace } from "../../lib/types";
  import AnalysisMap, { type ColorBy } from "../components/AnalysisMap.svelte";
  import TraceChart, { ChartGroup, type Series } from "../components/TraceChart.svelte";
  import { nav } from "../nav.svelte";

  const A = "#ff3341";
  const B = "#38bdf8";

  let tracks = $state<string[]>([]);
  let laps = $state<LapRecord[]>([]);
  let traceA = $state.raw<LapTrace | null>(null);
  let traceB = $state.raw<LapTrace | null>(null);
  let colorBy = $state<ColorBy>("speed");
  let cursorIdx = $state<number | null>(null);
  let loading = $state(false);
  let remoteError = $state("");
  const group = new ChartGroup();

  const units = $derived(config.value.general.units);
  const lapA = $derived(laps.find((l) => l.id === nav.lapA) ?? null);
  const lapB = $derived(laps.find((l) => l.id === nav.lapB) ?? null);

  onMount(async () => {
    tracks = (await api.getFilters()).tracks;
    if (!nav.track && tracks.length) nav.track = tracks[0];
  });

  // Laps for the selected track.
  $effect(() => {
    const track = nav.track;
    if (!track) return;
    api.listTrackLaps(track, null, false, 1000).then((l) => {
      if (nav.track !== track) return;
      laps = l;
      if (nav.lapA == null || !l.some((x) => x.id === nav.lapA)) {
        nav.lapA = l.find((x) => x.valid && !x.outLap && !x.inLap)?.id ?? l[0]?.id ?? null;
        nav.lapB = null;
      }
    });
  });

  $effect(() => {
    const a = nav.lapA;
    const b = nav.lapB;
    const remote = nav.remoteB;
    loading = true;
    remoteError = "";
    const loadB = b != null ? api.getLapTrace(b) : remote ? api.rankedLapTrace(remote.id) : null;
    Promise.all([
      a != null ? api.getLapTrace(a) : null,
      // A ranked lap may be gone or the server unreachable; still show lap A.
      Promise.resolve(loadB).catch((e) => {
        remoteError = String(e);
        return null;
      }),
    ]).then(([ta, tb]) => {
      if (nav.lapA !== a || nav.lapB !== b || nav.remoteB !== remote) return;
      traceA = ta;
      traceB = tb;
      loading = false;
      cursorIdx = null;
    });
  });

  /** Lap B's select value: a local lap id, "remote" or "" (none). */
  const bValue = $derived(nav.remoteB ? "remote" : nav.lapB != null ? String(nav.lapB) : "");
  function selectB(value: string) {
    if (value === "remote") return;
    nav.remoteB = null;
    nav.lapB = value ? Number(value) : null;
  }

  const data: Aligned | null = $derived(traceA && traceA.d.length > 10 ? align(traceA, traceB, 2) : null);
  const zonesA: BrakeZone[] = $derived(data ? brakeZones(data.a, data.d) : []);
  const zonesB: BrakeZone[] = $derived(data?.b ? brakeZones(data.b, data.d) : []);

  // Charts. Each `series` array is rebuilt only when the aligned data changes.
  const sp = (arr: Float64Array) => Array.from(arr, (v) => toSpeed(v, units));
  const pct = (arr: Float64Array) => Array.from(arr, (v) => v * 100);

  const charts = $derived.by(() => {
    if (!data) return [];
    const { a, b } = data;
    const list: { title: string; series: Series[]; height?: number; yRange?: [number, number]; format?: (v: number) => string }[] = [];
    if (data.delta) {
      list.push({
        title: "Delta (A − B, s)",
        series: [{ label: "Δ", data: data.delta, color: "#e8eaf0", fill: "rgb(232 234 240 / 0.06)" }],
        height: 110,
        format: (v) => (isFinite(v) ? (v > 0 ? "+" : "") + v.toFixed(3) : ""),
      });
    }
    list.push({
      title: `Speed (${speedUnit(units)})`,
      series: [
        { label: "A", data: sp(a.speed), color: A },
        ...(b ? [{ label: "B", data: sp(b.speed), color: B }] : []),
      ],
      height: 170,
    });
    list.push({
      title: "Throttle (%)",
      series: [
        { label: "A", data: pct(a.throttle), color: "#2fbf71" },
        ...(b ? [{ label: "B", data: pct(b.throttle), color: B, dash: [4, 3] }] : []),
      ],
      yRange: [0, 102],
    });
    list.push({
      title: "Brake (%)",
      series: [
        { label: "A", data: pct(a.brake), color: "#e5484d" },
        ...(b ? [{ label: "B", data: pct(b.brake), color: B, dash: [4, 3] }] : []),
      ],
      yRange: [0, 102],
    });
    list.push({
      title: "Steering (%)",
      series: [
        { label: "A", data: pct(a.steering), color: A },
        ...(b ? [{ label: "B", data: pct(b.steering), color: B }] : []),
      ],
      yRange: [-100, 100],
    });
    list.push({
      title: "Gear",
      series: [
        { label: "A", data: a.gear, color: A, stepped: true },
        ...(b ? [{ label: "B", data: b.gear, color: B, stepped: true }] : []),
      ],
      height: 100,
    });
    list.push({
      title: "RPM",
      series: [
        { label: "A", data: a.rpm, color: A },
        ...(b ? [{ label: "B", data: b.rpm, color: B }] : []),
      ],
      height: 110,
    });
    list.push({
      title: "Tyre surface temperature, lap A",
      series: [
        { label: "FL", data: Array.from(a.tyreFl, (v) => temp(v, units)), color: "#4c9aff" },
        { label: "FR", data: Array.from(a.tyreFr, (v) => temp(v, units)), color: "#2fbf71" },
        { label: "RL", data: Array.from(a.tyreRl, (v) => temp(v, units)), color: "#f2a33a" },
        { label: "RR", data: Array.from(a.tyreRr, (v) => temp(v, units)), color: "#e5484d" },
      ],
      height: 120,
    });
    return list;
  });

  const readout = $derived.by(() => {
    if (!data || cursorIdx == null) return null;
    const i = cursorIdx;
    const row = (c: Aligned["a"]) => ({
      t: c.t[i],
      speed: toSpeed(c.speed[i], units),
      gear: Math.round(c.gear[i]),
      throttle: c.throttle[i] * 100,
      brake: c.brake[i] * 100,
      steering: c.steering[i] * 100,
      gLat: c.gLat[i],
    });
    return { d: data.d[i], a: row(data.a), b: data.b ? row(data.b) : null, delta: data.delta?.[i] ?? null };
  });

  /** Pair each braking zone of A with B's zone that starts within 80 m. */
  const corners = $derived(
    zonesA.map((za, k) => {
      const da = data!.d[za.start];
      const zb = zonesB.find((z) => Math.abs(data!.d[z.start] - da) < 80) ?? null;
      return {
        n: k + 1,
        za,
        zb,
        // Metres; positive means A starts braking before B.
        brakeDiff: zb ? data!.d[zb.start] - da : null,
        minA: toSpeed(za.minSpeed, units),
        minB: zb ? toSpeed(zb.minSpeed, units) : null,
        deltaGain:
          data!.delta && zb
            ? data!.delta[Math.min(data!.d.length - 1, Math.max(za.end, zb.end) + 50)] - data!.delta[Math.max(0, Math.min(za.start, zb.start) - 25)]
            : null,
      };
    }),
  );

  function lapLabel(l: LapRecord) {
    const flags = [!l.valid && "invalid", l.outLap && "out", l.inLap && "in"].filter(Boolean).join(", ");
    return `${lapTime(l.lapTime)} · L${l.lapNumber} · ${l.car} · ${dateTime(l.recordedAt)}${flags ? ` (${flags})` : ""}`;
  }

  const COLOR_MODES: { id: ColorBy; label: string }[] = [
    { id: "speed", label: "Speed" },
    { id: "inputs", label: "Inputs" },
    { id: "gear", label: "Gear" },
    { id: "delta", label: "Time gain" },
  ];
</script>

<div class="p-5">
  <div class="mb-4 flex flex-wrap items-end gap-3">
    <label class="flex flex-col gap-1 text-xs text-muted">
      Track
      <select class="min-w-[200px] field text-sm text-fg" bind:value={nav.track}>
        {#each tracks as t}<option value={t}>{t}</option>{/each}
      </select>
    </label>
    <label class="flex flex-col gap-1 text-xs text-muted">
      <span class="flex items-center gap-1.5"><span class="h-2 w-2 rounded-full" style:background={A}></span>Lap A</span>
      <select class="w-[360px] field text-sm text-fg" bind:value={nav.lapA}>
        {#each laps as l (l.id)}<option value={l.id}>{lapLabel(l)}</option>{/each}
      </select>
    </label>
    <label class="flex flex-col gap-1 text-xs text-muted">
      <span class="flex items-center gap-1.5"><span class="h-2 w-2 rounded-full" style:background={B}></span>Compare with (B)</span>
      <select class="w-[360px] field text-sm text-fg" value={bValue} onchange={(e) => selectB(e.currentTarget.value)}>
        <option value="">None</option>
        {#if nav.remoteB}
          <option value="remote">Ranked: {lapTime(nav.remoteB.lapTime)} · {nav.remoteB.label}</option>
        {/if}
        {#each laps as l (l.id)}{#if l.id !== nav.lapA}<option value={String(l.id)}>{lapLabel(l)}</option>{/if}{/each}
      </select>
    </label>
    {#if nav.lapB != null}
      <button
        class="btn"
        onclick={() => ([nav.lapA, nav.lapB] = [nav.lapB, nav.lapA])}>Swap</button
      >
    {/if}
    {#if laps.length && !nav.lapB && !nav.remoteB}
      <button
        class="btn"
        title="Compare with your fastest clean lap on this track"
        onclick={() => (nav.lapB = laps.find((l) => l.valid && !l.outLap && !l.inLap && l.id !== nav.lapA)?.id ?? null)}
        >vs best lap</button
      >
    {/if}
  </div>

  {#if remoteError}
    <p class="mb-3 text-sm text-bad">Couldn't load the ranked lap: {remoteError}</p>
  {/if}

  {#if nav.remoteB && nav.track && !tracks.includes(nav.track)}
    <div class="card p-10 text-center text-muted">You have no laps on {nav.track} yet. Drive some to compare with this ranked lap.</div>
  {:else if !tracks.length}
    <div class="card p-10 text-center text-muted">No laps recorded yet.</div>
  {:else if !data}
    <div class="card p-10 text-center text-muted">{loading ? "Loading…" : "Select a lap."}</div>
  {:else}
    <div class="grid grid-cols-[minmax(0,1fr)_440px] gap-4">
      <div class="flex min-w-0 flex-col gap-2">
        {#each charts as c (c.title)}
          <TraceChart
            title={c.title}
            x={data.d}
            series={c.series}
            height={c.height}
            yRange={c.yRange}
            format={c.format}
            {group}
            syncKey="analysis"
            {cursorIdx}
            oncursor={(i) => (cursorIdx = i)}
          />
        {/each}
        <div class="px-1 text-xs text-faint">Drag across a chart to zoom, double-click to reset.</div>
      </div>

      <div class="flex flex-col gap-3">
        <div class="sticky top-0 flex flex-col gap-3">
          <div class="flex gap-1 rounded-lg bg-surface p-1 text-sm">
            {#each COLOR_MODES as m}
              <button
                class={["flex-1 rounded-md py-1", colorBy === m.id ? "bg-surface-3 text-fg" : "text-muted hover:text-fg"]}
                disabled={m.id === "delta" && !data.delta}
                onclick={() => (colorBy = m.id)}>{m.label}</button
              >
            {/each}
          </div>
          <AnalysisMap {data} {colorBy} zones={zonesA} {cursorIdx} oncursor={(i) => (cursorIdx = i)} height={400} />

          <div class="card p-3 text-sm tnum">
            {#if readout}
              <div class="mb-2 flex justify-between text-xs text-muted">
                <span>{Math.round(readout.d)} m</span>
                {#if readout.delta != null}
                  <span class={readout.delta > 0 ? "text-bad" : "text-good"}>Δ {fmtDelta(readout.delta)}</span>
                {/if}
              </div>
              <div class="grid grid-cols-[60px_repeat(6,1fr)] gap-x-2 gap-y-1 text-right">
                <span></span><span class="text-xs text-muted">Time</span><span class="text-xs text-muted">Speed</span><span class="text-xs text-muted">Gear</span><span class="text-xs text-muted">Thr</span><span class="text-xs text-muted">Brk</span><span class="text-xs text-muted">Steer</span>
                {#each [["A", readout.a, A], ...(readout.b ? [["B", readout.b, B]] : [])] as [label, r, color]}
                  {@const row = r as NonNullable<typeof readout>["a"]}
                  <span class="text-left font-semibold" style:color={color as string}>{label}</span>
                  <span>{lapTime(row.t, 2)}</span>
                  <span>{Math.round(row.speed)}</span>
                  <span>{row.gear}</span>
                  <span class="text-throttle">{Math.round(row.throttle)}%</span>
                  <span class="text-brake">{Math.round(row.brake)}%</span>
                  <span>{Math.round(row.steering)}%</span>
                {/each}
              </div>
            {:else}
              <div class="text-center text-xs text-muted">Hover the charts or the map to inspect inputs.</div>
            {/if}
          </div>

          <div class="grid grid-cols-2 gap-3 text-sm tnum">
            {#each [[lapA, A, "A"], [lapB, B, "B"]] as [l, color, name]}
              {@const lap = l as LapRecord | null}
              {#if lap}
                <div class="card p-3">
                  <div class="text-xs font-semibold" style:color={color as string}>Lap {name}</div>
                  <div class={["font-display text-xl font-semibold", !lap.valid && "text-bad"]}>{lapTime(lap.lapTime)}</div>
                  <div class="text-xs text-muted">{sectorTime(lap.s1)} · {sectorTime(lap.s2)} · {sectorTime(lap.s3)}</div>
                  <div class="text-xs text-muted">Top {Math.round(toSpeed(lap.maxSpeed, units))} {speedUnit(units)}</div>
                </div>
              {/if}
            {/each}
            {#if nav.remoteB && traceB}
              {@const r = nav.remoteB}
              <div class="card p-3">
                <div class="truncate text-xs font-semibold" style:color={B}>Lap B · {r.label}</div>
                <div class="font-display text-xl font-semibold">{lapTime(r.lapTime)}</div>
                <div class="text-xs text-muted">{sectorTime(r.s1)} · {sectorTime(r.s2)} · {sectorTime(r.s3)}</div>
                <div class="text-xs text-muted">Top {Math.round(toSpeed(Math.max(...traceB.speed), units))} {speedUnit(units)}</div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>

    {#if corners.length}
      <h2 class="mt-6 mb-2 section-title">Braking zones</h2>
      <div class="overflow-hidden rounded-lg border border-line">
        <table class="w-full text-sm tnum">
          <thead class="bg-surface-2 text-left text-xs text-muted">
            <tr>
              <th class="px-3 py-2 font-medium">#</th>
              <th class="px-3 py-2 font-medium">Brake point A</th>
              <th class="px-3 py-2 font-medium">vs B</th>
              <th class="px-3 py-2 font-medium">Entry speed A</th>
              <th class="px-3 py-2 font-medium">Min speed A</th>
              <th class="px-3 py-2 font-medium">Min speed B</th>
              <th class="px-3 py-2 font-medium">Time through zone</th>
            </tr>
          </thead>
          <tbody>
            {#each corners as c (c.n)}
              <tr class="cursor-pointer border-t border-line hover:bg-surface-2" onclick={() => (cursorIdx = c.za.start)}>
                <td class="px-3 py-1.5 font-semibold">{c.n}</td>
                <td class="px-3 py-1.5">{Math.round(data.d[c.za.start])} m</td>
                <td class="px-3 py-1.5">
                  {#if c.brakeDiff != null}
                    <!-- brakeDiff > 0: B brakes further down the road, i.e. A brakes earlier. -->
                    <span class={c.brakeDiff > 0 ? "text-warn" : "text-good"}>
                      {Math.abs(Math.round(c.brakeDiff))} m {c.brakeDiff > 0 ? "earlier" : "later"}
                    </span>
                  {:else}–{/if}
                </td>
                <td class="px-3 py-1.5">{Math.round(toSpeed(c.za.entrySpeed, units))}</td>
                <td class="px-3 py-1.5">{Math.round(c.minA)}</td>
                <td class="px-3 py-1.5">{c.minB != null ? Math.round(c.minB) : "–"}</td>
                <td class="px-3 py-1.5">
                  {#if c.deltaGain != null}
                    <span class={c.deltaGain > 0 ? "text-bad" : "text-good"}>{fmtDelta(c.deltaGain)}</span>
                  {:else}–{/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>
