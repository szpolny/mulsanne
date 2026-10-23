<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { dateTime, lapTime } from "../../lib/format";
  import type { Filters, LapRecord, SessionSummary } from "../../lib/types";
  import { config } from "../../lib/config.svelte";
  import { uploadable, uploads } from "../../lib/uploads.svelte";
  import LapTable from "../components/LapTable.svelte";
  import { nav } from "../nav.svelte";

  let filters = $state<Filters>({ tracks: [], cars: [] });
  let track = $state<string>("");
  let car = $state<string>("");
  let sessions = $state<SessionSummary[]>([]);
  let laps = $state<LapRecord[]>([]);
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

  onMount(async () => {
    filters = await api.getFilters();
    await loadSessions();
  });

  $effect(() => {
    loadLaps(nav.sessionId);
  });

  function toggle(l: LapRecord) {
    if (selected.includes(l.id)) selected = selected.filter((x) => x !== l.id);
    else selected = [...selected, l.id].slice(-2);
  }

  async function deleteSession() {
    if (!current || !confirm(`Delete this session and its ${current.lapCount} laps?`)) return;
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
    const online = uploads.byLap[l.id]?.remoteId ? "\nIt stays on the online rankings; remove it there first if you want it gone." : "";
    if (!confirm(`Delete lap ${l.lapNumber} (${lapTime(l.lapTime)})?${online}`)) return;
    await api.deleteLap(l.id);
    await loadLaps(nav.sessionId);
    await loadSessions();
  }
</script>

<div class="flex h-full">
  <aside class="flex w-[340px] shrink-0 flex-col border-r border-line">
    <div class="flex gap-2 border-b border-line p-3">
      <select class="min-w-0 flex-1 field text-sm" bind:value={track} onchange={loadSessions}>
        <option value="">All tracks</option>
        {#each filters.tracks as t}<option value={t}>{t}</option>{/each}
      </select>
      <select class="min-w-0 flex-1 field text-sm" bind:value={car} onchange={loadSessions}>
        <option value="">All cars</option>
        {#each filters.cars as c}<option value={c}>{c}</option>{/each}
      </select>
    </div>
    <div class="flex-1 overflow-y-auto">
      {#each sessions as s (s.id)}
        <button
          class={["block w-full border-b border-line px-4 py-3 text-left", nav.sessionId === s.id ? "bg-surface-3" : "hover:bg-surface-2"]}
          onclick={() => (nav.sessionId = s.id)}
        >
          <div class="flex items-baseline justify-between gap-2">
            <span class="truncate font-medium">{s.track}</span>
            <span class="shrink-0 text-sm font-semibold text-best tnum">{lapTime(s.bestLap)}</span>
          </div>
          <div class="truncate text-xs text-muted">{s.car} · {s.sessionType}{s.source === "demo" ? " · demo" : ""}</div>
          <div class="text-xs text-faint">{dateTime(s.startedAt)} · {s.lapCount} laps</div>
        </button>
      {:else}
        <div class="p-6 text-center text-sm text-muted">No sessions yet. Drive some laps!</div>
      {/each}
    </div>
  </aside>

  <section class="min-w-0 flex-1 overflow-y-auto p-6">
    {#if current}
      <div class="mb-4 flex items-start justify-between gap-4">
        <div>
          <h1 class="page-title">{current.track}</h1>
          <div class="text-sm text-muted">
            {current.car} ({current.carClass}) · {current.sessionType} · {dateTime(current.startedAt)}{current.gameVersion
              ? ` · game version ${current.gameVersion}`
              : ""}
          </div>
        </div>
        <div class="flex gap-2">
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
      {#if uploadNote}<p class="mb-2 text-xs text-bad">{uploadNote}</p>{/if}
      <p class="mb-2 text-xs text-muted">Click laps to select (A, then B) for comparison.</p>
      <LapTable
        {laps}
        {selected}
        onselect={toggle}
        onopen={(l) => nav.openAnalysis(l.track, l.id)}
        ondelete={deleteLap}
        {fromGame}
      />
    {:else}
      <div class="p-10 text-center text-muted">Select a session.</div>
    {/if}
  </section>
</div>
