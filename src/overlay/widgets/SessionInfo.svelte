<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { classColor, clock, sessionLabel, timeOfDay } from "../../lib/format";
  import { live } from "../../lib/live.svelte";

  let {}: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const snap = $derived(live.snap);
  const s = $derived(snap?.session);
  const me = $derived(snap?.vehicles.find((v) => v.isPlayer));
  const lapLimited = $derived(!!s && s.maxLaps > 0 && s.maxLaps < 10000 && s.endEt <= 0);
  const classSize = $derived(me ? (snap?.vehicles.filter((v) => v.class === me.class).length ?? 0) : 0);
</script>

{#snippet stat(label: string, value: string)}
  <div>
    <div class="hud-label">{label}</div>
    <div class="hud-value mt-1 text-[22px]">{value}</div>
  </div>
{/snippet}

<div class="hud flex w-max items-center gap-5 px-4 py-2.5">
  {@render stat(
    s ? sessionLabel(s.kind) : "Session",
    !s ? "–" : lapLimited ? `${Math.min((me?.totalLaps ?? 0) + 1, s.maxLaps)} / ${s.maxLaps}` : clock(s.timeRemaining),
  )}
  {#if me}
    <div>
      <div class="hud-label flex items-center gap-1.5">
        <span class="h-2 w-2 rounded-full" style:background={classColor(me.class)}></span>{me.class}
      </div>
      <div class="hud-value mt-1 text-[22px]">P{me.classPlace}<span class="text-[14px] text-muted"> / {classSize}</span></div>
    </div>
    {@render stat("Overall", `P${me.place}`)}
  {/if}
  {@render stat("Laps", String(me?.totalLaps ?? 0))}
  {@render stat("Time", s ? timeOfDay(s.timeOfDay) : "–")}
</div>
