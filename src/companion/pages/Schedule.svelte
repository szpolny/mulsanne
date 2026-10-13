<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import TrackOutline from "../../lib/TrackOutline.svelte";
  import type { DailyRace } from "../../lib/types";

  const TIERS = ["beginner", "intermediate", "advanced"] as const;
  const TIER_COLOR: Record<string, string> = {
    beginner: "var(--color-good)",
    intermediate: "var(--color-warn)",
    advanced: "var(--color-accent-hi)",
  };

  let races = $state<DailyRace[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let fetchedAt = $state<number | null>(null);
  let now = $state(Date.now());
  let tier = $state<string | null>(null);

  async function load() {
    loading = true;
    try {
      races = await api.dailyRaces();
      error = null;
      fetchedAt = Date.now();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
    const tick = setInterval(() => (now = Date.now()), 15_000);
    const refresh = setInterval(load, 5 * 60_000);
    return () => {
      clearInterval(tick);
      clearInterval(refresh);
    };
  });

  /** Races with at least one start still ahead, soonest first. */
  const upcoming = $derived(
    races
      .map((r) => ({ ...r, starts: r.starts.filter((t) => t > now) }))
      .filter((r) => r.starts.length > 0 && (!tier || r.tier === tier))
      .sort((a, b) => a.starts[0] - b.starts[0]),
  );

  const timeFmt = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
  const dayFmt = new Intl.DateTimeFormat(undefined, { weekday: "short" });

  function startLabel(t: number): string {
    const sameDay = new Date(t).toDateString() === new Date(now).toDateString();
    return sameDay ? timeFmt.format(t) : `${dayFmt.format(t)} ${timeFmt.format(t)}`;
  }

  function untilLabel(t: number): string {
    const min = Math.max(0, Math.round((t - now) / 60_000));
    if (min < 1) return "starting now";
    if (min < 60) return `in ${min} min`;
    return `in ${Math.floor(min / 60)} h ${String(min % 60).padStart(2, "0")} min`;
  }
</script>

<div class="mx-auto max-w-5xl p-6">
  <div class="mb-5 flex items-start justify-between gap-4">
    <div>
      <h1 class="mb-1 page-title">Daily races</h1>
      <p class="text-sm text-muted">
        Upcoming Le Mans Ultimate daily races from RaceControl, in your local time. No game needed.
      </p>
    </div>
    <button class="btn shrink-0" disabled={loading} onclick={load}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={loading ? "animate-spin" : ""}><path d="M20 12a8 8 0 1 1-2.34-5.66 M20 4v4h-4" /></svg>
      {loading ? "Loading" : "Refresh"}
    </button>
  </div>

  <div class="mb-4 flex flex-wrap gap-1.5">
    {#each [null, ...TIERS] as t}
      <button
        class={[
          "rounded-full border px-3 py-1 text-xs font-semibold capitalize transition-colors",
          tier === t ? "border-accent bg-surface-3 text-fg" : "border-line text-muted hover:text-fg",
        ]}
        onclick={() => (tier = t)}
      >
        {t ?? "All tiers"}
      </button>
    {/each}
  </div>

  {#if error}
    <div class="card mb-4 border-bad/40 p-4 text-sm">
      <div class="font-semibold text-bad">Could not load the schedule</div>
      <div class="mt-1 text-muted">{error}</div>
    </div>
  {/if}

  <div class="flex flex-col gap-3">
    {#each upcoming as race (race.name + race.track)}
      <div class="card flex overflow-hidden">
        <div class="flex w-32 shrink-0 items-center justify-center border-r border-line bg-surface-2 p-3 text-fg/80">
          <TrackOutline track={race.track} class="h-20 w-full" />
        </div>
        <div class="min-w-0 flex-1 p-4">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <div class="mb-1 flex items-center gap-2 text-[11px] font-semibold tracking-wide uppercase" style:color={TIER_COLOR[race.tier] ?? "var(--color-muted)"}>
                <span class="h-1.5 w-1.5 rounded-full" style:background="currentColor"></span>
                {race.tier}
              </div>
              <div class="truncate font-display text-xl font-semibold">{race.name}</div>
              <div class="mt-0.5 text-sm text-muted">{race.track} · {race.duration}</div>
            </div>
            <div class="shrink-0 text-right tnum">
              <div class="text-xs text-muted">Next start</div>
              <div class="text-2xl font-semibold">{startLabel(race.starts[0])}</div>
              <div class="text-xs text-accent-hi">{untilLabel(race.starts[0])}</div>
            </div>
          </div>
          {#if race.starts.length > 1}
            <div class="mt-3 flex flex-wrap gap-1.5 tnum">
              {#each race.starts.slice(1) as t}
                <span class="rounded bg-surface-3 px-2 py-0.5 text-xs text-muted">{startLabel(t)}</span>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {:else}
      {#if !loading && !error}
        <div class="card p-10 text-center text-muted">
          No more daily races listed for today. RaceControl publishes the next day's slots after midnight UTC.
        </div>
      {/if}
    {/each}
  </div>

  {#if fetchedAt}
    <p class="mt-4 text-xs text-faint">
      Source: racecontrol.gg · updated {timeFmt.format(fetchedAt)} · refreshes every 5 minutes. RaceControl only lists the rest of the current UTC day.
      Track outlines from Outlap, f1-circuits (MIT) and © OpenStreetMap contributors (ODbL).
    </p>
  {/if}
</div>
