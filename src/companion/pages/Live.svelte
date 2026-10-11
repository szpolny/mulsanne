<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { api } from "../../lib/api";
  import { config } from "../../lib/config.svelte";
  import { delta, fuel, fuelUnit, lapTime, sessionLabel, speed, speedUnit } from "../../lib/format";
  import { live } from "../../lib/live.svelte";
  import type { LapRecord, LapRecordedEvent, SessionSummary } from "../../lib/types";
  import CarLogo from "../../overlay/CarLogo.svelte";
  import LapTable from "../components/LapTable.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { nav } from "../nav.svelte";

  const snap = $derived(live.snap);
  const p = $derived(snap?.player);
  const units = $derived(config.value.general.units);

  let session = $state<SessionSummary | null>(null);
  let laps = $state<LapRecord[]>([]);
  let toast = $state<string | null>(null);

  async function refresh(sessionId?: number) {
    const sessions = await api.listSessions({ limit: 1 });
    session = sessionId != null ? (sessions.find((s) => s.id === sessionId) ?? sessions[0] ?? null) : (sessions[0] ?? null);
    laps = session ? (await api.listLaps(session.id)).reverse() : [];
  }

  onMount(() => {
    refresh();
  });
  const unlisten = listen<LapRecordedEvent>("lap-recorded", (e) => {
    refresh(e.payload.sessionId);
    if (e.payload.personalBest) {
      toast = `New personal best: ${lapTime(e.payload.lapTime)}`;
      setTimeout(() => (toast = null), 6000);
    }
  });
  onDestroy(() => unlisten.then((f) => f()));
</script>

<div class="mx-auto max-w-6xl p-6">
  <div class="mb-5 flex items-center justify-between">
    <h1 class="page-title">Live</h1>
    {#if toast}
      <div class="rounded-full bg-best/15 px-3 py-1 text-sm font-semibold text-best">{toast}</div>
    {/if}
  </div>

  <div class="grid gap-4 lg:grid-cols-3">
    <section class="card p-5 lg:col-span-2">
      {#if !snap || snap.source === "none"}
        <div class="text-xl font-semibold">Waiting for Le Mans Ultimate</div>
        <p class="mt-2 max-w-xl text-sm text-muted">
          Start the game and get on track. Run it in <b class="text-fg">borderless</b> or windowed mode so the overlay can
          draw on top. No game plugin is needed: Mulsanne reads LMU's built-in shared memory.
        </p>
      {:else}
        <div class="flex items-end justify-between gap-6">
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <span class="rounded-full bg-accent/15 px-2.5 py-0.5 text-xs font-semibold text-accent-hi">{sessionLabel(snap.session.kind)}</span>
              {#if snap.source === "demo"}
                <span class="rounded-full bg-warn/15 px-2.5 py-0.5 text-xs font-semibold text-warn">Demo data</span>
              {/if}
            </div>
            <div class="mt-2 truncate text-2xl font-bold">{snap.session.track || "—"}</div>
            <div class="mt-1 flex items-center gap-2 text-sm text-muted">
              {#if p}
                <CarLogo model={p.vehicleModel} entry={p.vehicleName} size={15} />
                {p.vehicleModel || p.vehicleName} · {p.vehicleClass}
              {:else}Not in car{/if}
            </div>
          </div>
          {#if p}
            <div class="shrink-0 text-right">
              <div class="text-xs text-muted">Lap {p.lapNumber + 1}</div>
              <div class="font-display text-4xl leading-none font-semibold tnum">{lapTime(snap.timing.currentLapTime, 1)}</div>
            </div>
          {/if}
        </div>
        {#if p}
          <div class="mt-5 grid grid-cols-[repeat(auto-fit,minmax(110px,1fr))] gap-2 tnum">
            {#each [
              ["Speed", `${Math.round(speed(p.speed, units))}`, speedUnit(units)],
              ["Gear", p.gear < 0 ? "R" : p.gear === 0 ? "N" : String(p.gear), ""],
              ["Δ PB", delta(snap.timing.deltaPb), ""],
              ["Best", lapTime(snap.timing.sessionBest), ""],
              ["Fuel", fuel(p.fuel, units).toFixed(1), fuelUnit(units)],
            ] as [label, value, unit]}
              <div class="rounded-lg bg-surface-2 px-3 py-2.5">
                <div class="text-xs text-muted">{label}</div>
                <div class="mt-0.5 font-display text-2xl font-semibold whitespace-nowrap">{value} <span class="text-xs font-medium text-muted">{unit}</span></div>
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </section>

    <section class="flex flex-col gap-3 card p-4 text-sm">
      <div class="section-title text-sm text-muted">Overlay</div>
      <div class="flex items-center justify-between">
        <span>Show overlay</span>
        <Toggle checked={live.status.overlayVisible} onchange={(v) => api.setOverlayVisible(v)} label="Show overlay" />
      </div>
      <div class="flex items-center justify-between">
        <span>Demo data</span>
        <Toggle checked={live.status.demo} onchange={(v) => api.setDemoMode(v)} label="Demo data" />
      </div>
      <button
        class={["btn mt-1", live.status.editMode ? "bg-warn! text-black" : "btn-primary"]}
        onclick={() => api.setEditMode(!live.status.editMode)}
      >
        {live.status.editMode ? "Finish editing layout" : "Edit overlay layout"}
      </button>
      <div class="text-xs leading-relaxed text-muted">
        <kbd class="kbd">Ctrl+Shift+F9</kbd> edit layout<br />
        <kbd class="kbd">Ctrl+Shift+F10</kbd> show / hide
      </div>
    </section>
  </div>

  <div class="mt-6 mb-2 flex items-baseline justify-between">
    <h2 class="section-title">
      Latest session
      {#if session}<span class="ml-2 text-sm font-normal text-muted">{session.track} · {session.car} · {session.sessionType}</span>{/if}
    </h2>
    {#if session}
      <button class="text-sm text-accent hover:underline" onclick={() => { nav.sessionId = session!.id; nav.page = "sessions"; }}>Open session</button>
    {/if}
  </div>
  <LapTable {laps} onopen={(l) => nav.openAnalysis(l.track, l.id)} />
</div>
