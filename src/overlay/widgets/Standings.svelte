<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { classColor, clock, gap, lapTime, sessionLabel, shortName, timeOfDay } from "../../lib/format";
  import { useLive } from "../../lib/live.svelte";
  import type { Vehicle } from "../../lib/types";
  import CarLogo from "../CarLogo.svelte";
  import EnergyBadge from "../EnergyBadge.svelte";
  import TyreBadge from "../TyreBadge.svelte";

  const live = useLive();

  let { opts }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const snap = $derived(live.snap);
  const isRace = $derived(snap?.session.kind === "race");

  // Session header: lap-limited races count laps, everything else counts down the clock.
  const me = $derived(snap?.vehicles.find((v) => v.isPlayer));
  const remaining = $derived.by(() => {
    const s = snap?.session;
    if (!s) return "–";
    if (s.maxLaps > 0 && s.maxLaps < 10000 && s.endEt <= 0) {
      const leaderLaps = Math.max(0, ...(snap?.vehicles.map((v) => v.totalLaps) ?? []));
      return `${Math.min(leaderLaps + 1, s.maxLaps)} / ${s.maxLaps}`;
    }
    return clock(s.timeRemaining);
  });
  const classSize = $derived(me ? (snap?.vehicles.filter((v) => v.class === me.class).length ?? 0) : 0);
  // Backend estimate: exact for lap-limited sessions, from average lap time for timed ones.
  const lapsLeft = $derived.by(() => {
    const l = snap?.strategy.lapsRemaining ?? 0;
    return l > 0 && isFinite(l) ? `~${l.toFixed(1)}` : "–";
  });

  interface Row {
    v: Vehicle;
    gap: string;
    interval: string;
  }

  const groups = $derived.by(() => {
    if (!snap) return [];
    const myClass = snap.vehicles.find((v) => v.isPlayer)?.class;
    const byClass = new Map<string, Vehicle[]>();
    for (const v of [...snap.vehicles].sort((a, b) => a.place - b.place)) {
      if (opts.classes === "mine" && myClass != null && v.class !== myClass) continue;
      if (!byClass.has(v.class)) byClass.set(v.class, []);
      byClass.get(v.class)!.push(v);
    }
    const entries = [...byClass.entries()];
    if (opts.myClassFirst && myClass != null) entries.sort(([a], [b]) => Number(b === myClass) - Number(a === myClass));
    const around = Math.max(0, opts.around ?? 1);
    return entries.map(([cls, cars]) => {
      const leader = cars[0];
      const limit = Math.max(1, (cls === myClass ? opts.maxRows : opts.maxRowsOther) ?? 8);
      const bests = cars.map((v) => v.bestLap).filter((t) => t > 0);
      const classBest = bests.length ? Math.min(...bests) : 0;
      const rows: Row[] = cars.map((v, i) => {
        if (i === 0) return { v, gap: isRace ? "Leader" : lapTime(v.bestLap), interval: "" };
        const prev = cars[i - 1];
        if (!isRace) {
          const d = v.bestLap > 0 && leader.bestLap > 0 ? v.bestLap - leader.bestLap : 0;
          const di = v.bestLap > 0 && prev.bestLap > 0 ? v.bestLap - prev.bestLap : 0;
          return { v, gap: d > 0 ? `+${d.toFixed(3)}` : "–", interval: di > 0 ? `+${di.toFixed(3)}` : "" };
        }
        return {
          v,
          gap: gap(v.timeBehindLeader - leader.timeBehindLeader, v.lapsBehindLeader - leader.lapsBehindLeader),
          interval: gap(v.timeBehindLeader - prev.timeBehindLeader, v.lapsBehindLeader - prev.lapsBehindLeader),
        };
      });
      // Keep the player visible: top rows plus the cars around the player.
      let visible = rows.slice(0, limit);
      const pi = rows.findIndex((r) => r.v.isPlayer);
      if (pi >= limit) {
        const top = Math.max(1, limit - (2 * around + 1));
        let start = Math.max(top, pi - around);
        let end = Math.min(rows.length, pi + around + 1);
        // Not enough room for the whole window: drop cars behind first, then ahead.
        while (top + end - start > limit && end > pi + 1) end--;
        while (top + end - start > limit && start < pi) start++;
        visible = [...rows.slice(0, top), ...rows.slice(start, end)];
      }
      return { cls, count: cars.length, classBest, rows: visible };
    });
  });

  const columns = $derived(
    [
      "22px", // position
      "36px", // number
      opts.showLogo ? "18px" : "",
      "minmax(130px,1fr)", // driver
      opts.showTyres ? "18px" : "",
      opts.showEnergy ? "40px" : "",
      opts.showPits ? "36px" : "",
      opts.showLast ? "70px" : "",
      opts.showBest ? "70px" : "",
      "64px", // gap
      opts.showInterval ? "56px" : "",
    ]
      .filter(Boolean)
      .join(" "),
  );

  /** Lap time highlight: purple = class best, green = the driver's personal best (only for last laps). */
  function tone(t: number, v: Vehicle, classBest: number, personal: boolean): string | undefined {
    if (!opts.colorTimes || t <= 0) return undefined;
    if (classBest > 0 && t <= classBest + 1e-4) return "var(--color-best)";
    if (personal && v.bestLap > 0 && t <= v.bestLap + 1e-4) return "var(--color-good)";
    return undefined;
  }
</script>

{#snippet time(t: number, color: string | undefined, cls: string)}
  <span class="text-right">
    <span
      class={["rounded-[4px] px-1", cls, color && "font-semibold text-white opacity-100!"]}
      style:background={color}>{lapTime(t)}</span
    >
  </span>
{/snippet}

{#snippet stat(label: string, value: string)}
  <span class="flex items-baseline gap-1.5">
    <span class="hud-label">{label}</span>
    <span class="hud-value text-[14px]">{value}</span>
  </span>
{/snippet}

<div class="hud w-max min-w-[380px] py-1.5 text-[14px]">
  {#if opts.showSession && snap}
    <div class="mx-1.5 mb-1 flex items-baseline gap-3.5 border-b border-white/10 px-1.5 pb-1">
      {@render stat(sessionLabel(snap.session.kind), remaining)}
      {#if me}
        <span class="flex items-baseline gap-1.5">
          <span class="h-2 w-2 self-center rounded-full" style:background={classColor(me.class)}></span>
          <span class="hud-value text-[14px]">P{me.classPlace}<span class="text-[11px] text-muted">/{classSize}</span></span>
        </span>
        {@render stat("Ovr", `P${me.place}`)}
      {/if}
      {@render stat("Laps", String(me?.totalLaps ?? 0))}
      {@render stat("Left", lapsLeft)}
      <span class="ml-auto hud-value text-[14px]">{timeOfDay(snap.session.timeOfDay)}</span>
    </div>
  {/if}
  {#each groups as g (g.cls)}
    <div class="flex items-center gap-2 px-3 pt-1.5 pb-1">
      <span class="h-2.5 w-2.5 rounded-full" style:background={classColor(g.cls)}></span>
      <span class="hud-title text-fg!">{g.cls || "Class"}</span>
      <span class="text-[12px] text-faint">{g.count}</span>
    </div>
    {#each g.rows as r, i (r.v.id)}
      <div
        class={[
          "mx-1.5 grid h-[26px] items-center gap-x-2 rounded-md px-1.5",
          r.v.isPlayer ? "bg-accent text-white" : i % 2 === 1 && "bg-white/[0.03]",
          (r.v.finishStatus === 2 || r.v.finishStatus === 3) && "opacity-50",
        ]}
        style:grid-template-columns={columns}
      >
        <span class="text-right font-semibold">{r.v.classPlace}</span>
        <span
          class="rounded-[4px] text-center text-[12px] leading-[18px] font-bold text-white"
          style:background={classColor(r.v.class)}
          style:box-shadow={r.v.isPlayer ? "0 0 0 1px rgb(255 255 255 / 0.6)" : undefined}>{r.v.number || "–"}</span
        >
        {#if opts.showLogo}<CarLogo model={r.v.model} entry={r.v.vehicle} />{/if}
        <span class="flex min-w-0 items-center gap-1.5">
          <span class="truncate font-medium">{shortName(r.v.driver)}</span>
          {#if r.v.finishStatus === 1}
            <span class="checkered h-3.5 w-5 shrink-0 rounded-[3px] ring-1 ring-white/40" style="--check: 3.5px" title="Finished"></span>
          {:else if r.v.finishStatus === 2}
            <span class="rounded-[4px] bg-white/15 px-1 text-[10px] leading-4 font-bold text-muted">DNF</span>
          {:else if r.v.finishStatus === 3}
            <span class="rounded-[4px] bg-bad px-1 text-[10px] leading-4 font-bold text-white">DQ</span>
          {:else if r.v.inGarage}
            <span class="rounded-[4px] bg-white/15 px-1 text-[10px] leading-4 font-bold text-fg">GAR</span>
          {:else if r.v.inPits}
            <span class="rounded-[4px] bg-white px-1 text-[10px] leading-4 font-bold text-black">PIT</span>
          {/if}
        </span>
        {#if opts.showTyres}
          <TyreBadge compound={r.v.tyreCompound} wear={r.v.tyreWear} laps={r.v.tyreLaps} />
        {/if}
        {#if opts.showEnergy}<EnergyBadge value={r.v.virtualEnergy} />{/if}
        {#if opts.showPits}
          <span class="text-right text-[13px]"><span class="text-[10px] opacity-50">PS</span> {r.v.numPitstops}</span>
        {/if}
        {#if opts.showLast}{@render time(r.v.lastLap, tone(r.v.lastLap, r.v, g.classBest, true), "text-[13px] opacity-75")}{/if}
        {#if opts.showBest}{@render time(r.v.bestLap, tone(r.v.bestLap, r.v, g.classBest, false), "text-[13px] opacity-75")}{/if}
        {#if !isRace && i === 0}
          {@render time(r.v.bestLap, tone(r.v.bestLap, r.v, g.classBest, false), "font-medium")}
        {:else}
          <span class="text-right font-medium">{r.gap}</span>
        {/if}
        {#if opts.showInterval}<span class="text-right text-[13px] opacity-65">{r.interval}</span>{/if}
      </div>
    {/each}
  {:else}
    <div class="px-3 py-2 text-muted">No session</div>
  {/each}
</div>
