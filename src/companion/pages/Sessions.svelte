<script lang="ts">
  import Select from "../components/Select.svelte";
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { classLabel, dateTime, lapTime } from "../../lib/format";
  import type { Filters, LapRecord, ResultEntry, SessionSummary } from "../../lib/types";
  import { config } from "../../lib/config.svelte";
  import { confirm } from "../../lib/confirm.svelte";
  import { uploadable, uploads } from "../../lib/uploads.svelte";
  import CarRender from "../../lib/CarRender.svelte";
  import TrackHero from "../../lib/TrackHero.svelte";
  import LapTable from "../components/LapTable.svelte";
  import ResultsTable from "../components/ResultsTable.svelte";
  import { nav } from "../nav.svelte";

  let filters = $state<Filters>({ tracks: [], cars: [] });
  let track = $state<string>("");
  let car = $state<string>("");
  let sessions = $state<SessionSummary[]>([]);
  let laps = $state<LapRecord[]>([]);
  let results = $state<ResultEntry[]>([]);
  let selected = $state<number[]>([]);

  const current = $derived(sessions.find((s) => s.id === nav.sessionId) ?? null);

  async function loadSessions() {
    sessions = await api.listSessions({ track: track || null, car: car || null, limit: 500 });
    if (nav.sessionId == null || !sessions.some((s) => s.id === nav.sessionId)) nav.sessionId = sessions[0]?.id ?? null;
  }

  async function loadLaps(id: number | null) {
    selected = [];
    laps = id == null ? [] : await api.listLaps(id);
  }

  async function loadResults(id: number | null) {
    const saved = sessions.find((s) => s.id === id)?.results;
    results = id == null || !saved ? [] : await api.sessionResults(id);
  }

  onMount(async () => {
    filters = await api.getFilters();
    await loadSessions();
  });

  $effect(() => {
    loadLaps(nav.sessionId);
  });

  $effect(() => {
    loadResults(current?.id ?? null);
  });

  function toggle(l: LapRecord) {
    if (selected.includes(l.id)) selected = selected.filter((x) => x !== l.id);
    else selected = [...selected, l.id].slice(-2);
  }

  async function deleteSession() {
    if (!current) return;
    const ok = await confirm({
      title: "Delete this session?",
      message: `${current.track} · ${current.car}. All ${current.lapCount} laps and their telemetry${current.results ? ", and the race results," : ""} will be removed from this PC. This can't be undone.`,
      confirmLabel: "Delete session",
      danger: true,
    });
    if (!ok) return;
    await api.deleteSession(current.id);
    nav.sessionId = null;
    await loadSessions();
  }

  const fromGame = $derived(current?.source === "lmu");
  const toUpload = $derived(fromGame ? laps.filter((l) => uploadable(l) && !uploads.byLap[l.id]) : []);
  let uploadNote = $state("");

  async function uploadSession() {
    uploadNote = "";
    try {
      const queued = await api.uploadLaps(toUpload.map((l) => l.id));
      if (queued === 0) uploadNote = "No laps in this session can be uploaded.";
    } catch (e) {
      uploadNote = String(e);
    }
  }

  async function deleteLap(l: LapRecord) {
    const ok = await confirm({
      title: `Delete lap ${l.lapNumber}?`,
      message: `The ${lapTime(l.lapTime)} lap and its telemetry will be removed from this PC. This can't be undone.`,
      note: uploads.byLap[l.id]?.remoteId ? "It stays on the online rankings; remove it there first if you want it gone." : undefined,
      confirmLabel: "Delete lap",
      danger: true,
    });
    if (!ok) return;
    await api.deleteLap(l.id);
    await loadLaps(nav.sessionId);
    await loadSessions();
  }
</script>

<div class="flex h-full">
  <aside class="flex w-[320px] shrink-0 flex-col border-r border-line bg-surface/40">
    <div class="flex flex-col gap-3 border-b border-line p-4">
      <div class="eyebrow">Sessions <span class="text-faint">{sessions.length || ""}</span></div>
      <div class="flex gap-2">
        <Select
          class="min-w-0 flex-1 text-sm"
          bind:value={track}
          onchange={loadSessions}
          label="Track"
          options={[{ value: "", label: "All tracks" }, ...filters.tracks.map((t) => ({ value: t, label: t }))]}
        />
        <Select
          class="min-w-0 flex-1 text-sm"
          bind:value={car}
          onchange={loadSessions}
          label="Car"
          options={[{ value: "", label: "All cars" }, ...filters.cars.map((c) => ({ value: c, label: c }))]}
        />
      </div>
    </div>
    <div class="flex-1 overflow-y-auto">
      {#each sessions as s (s.id)}
        <button
          class={["block w-full border-b border-line px-4 py-3 text-left", nav.sessionId === s.id ? "bg-surface-3" : "hover:bg-surface-2"]}
          onclick={() => (nav.sessionId = s.id)}
        >
          <div class="flex items-baseline justify-between gap-2">
            <span class="truncate font-display text-[17px] font-semibold tracking-[0.02em] uppercase">{s.track}</span>
            <span class="shrink-0 text-sm font-semibold text-best tnum">{lapTime(s.bestLap)}</span>
          </div>
          <div class="flex items-center gap-2">
            <div class="min-w-0 flex-1">
              <div class="truncate text-xs text-muted">{s.car} · {s.sessionType}{s.source === "demo" ? " · demo" : ""}</div>
              <div class="text-xs text-faint">
                {dateTime(s.startedAt)} · {s.lapCount} laps
                {#if s.position != null}
                  · <span class="font-semibold text-soft">P{s.position}</span>
                  {#if s.classPosition != null && s.classPosition !== s.position}(P{s.classPosition} in class){/if}
                {/if}
              </div>
            </div>
            <CarRender file={s.vehFile} view="thumb" class="h-9 w-[90px] shrink-0 object-contain" />
          </div>
        </button>
      {:else}
        <div class="empty-state text-sm">No sessions yet. Drive some laps!</div>
      {/each}
    </div>
  </aside>

  <section class="pane min-w-0 flex-1 overflow-y-auto">
    {#if current}
      <div class="card mb-6 overflow-hidden">
        <TrackHero track={current.track} file={current.vehFile} alt={current.car}>
          <h1 class="truncate font-display text-[24px] leading-tight font-bold uppercase">{current.track}</h1>
          <div class="mt-1 text-sm text-soft">
            {current.car} ({classLabel(current.carClass)}) · {current.sessionType}
          </div>
          <div class="text-sm text-faint">
            {dateTime(current.startedAt)}{current.gameVersion ? ` · game version ${current.gameVersion}` : ""}
          </div>
        </TrackHero>
      </div>
      {#if results.length}
        <ResultsTable {results} complete={current.results === "complete"} />
        <h2 class="eyebrow mb-3">Your laps</h2>
      {/if}
      {#if uploadNote}<p class="mb-2 text-xs text-bad">{uploadNote}</p>{/if}
      <div class="mb-3 flex items-center justify-between gap-4">
        <p class="text-sm text-muted">Click laps to select (A, then B) for comparison.</p>
        <div class="flex shrink-0 gap-2">
          <button
            class="btn btn-primary"
            disabled={selected.length === 0}
            onclick={() => nav.openAnalysis(current!.track, selected[0], selected[1] ?? null)}
          >
            {selected.length === 2 ? "Compare A vs B" : "Analyse lap"}
          </button>
          {#if config.value.online.enabled && fromGame}
            <button class="btn" disabled={toUpload.length === 0} onclick={uploadSession}>
              {toUpload.length ? `Upload ${toUpload.length} lap${toUpload.length === 1 ? "" : "s"}` : "Nothing to upload"}
            </button>
          {/if}
          <button class="btn hover:text-bad" onclick={deleteSession}>Delete session</button>
        </div>
      </div>
      <LapTable
        {laps}
        {selected}
        onselect={toggle}
        onopen={(l) => nav.openAnalysis(l.track, l.id)}
        ondelete={deleteLap}
        {fromGame}
      />
    {:else}
      <div class="card empty-state">Select a session.</div>
    {/if}
  </section>
</div>
