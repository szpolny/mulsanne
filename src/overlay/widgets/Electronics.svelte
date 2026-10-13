<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { useLive } from "../../lib/live.svelte";

  const live = useLive();

  let {}: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const p = $derived(live.snap?.player);
  const MOTOR = ["n/a", "Idle", "Deploy", "Regen"];
  const v = (val: number, max: number) => (max > 0 ? `${val}/${max}` : "–");
</script>

<div class="hud w-[200px] p-2.5 text-[13px]">
  {#if p}
    <div class="grid grid-cols-[1fr_auto] gap-y-0.5">
      <span class="text-muted">TC</span>
      <span class="font-semibold" class:text-warn={p.tcActive}>{v(p.tc, p.tcMax)}</span>
      <span class="text-muted">TC slip / cut</span>
      <span class="font-semibold">{p.tcMax > 0 ? `${p.tcSlip} / ${p.tcCut}` : "–"}</span>
      <span class="text-muted">ABS</span>
      <span class="font-semibold" class:text-warn={p.absActive}>{v(p.abs, p.absMax)}</span>
      <span class="text-muted">Motor map</span>
      <span class="font-semibold">{v(p.motorMap, p.motorMapMax)}</span>
      <span class="text-muted">Migration</span>
      <span class="font-semibold">{p.migration || "–"}</span>
      <span class="text-muted">ARB F / R</span>
      <span class="font-semibold">{p.frontArb} / {p.rearArb}</span>
      <span class="text-muted">Brake bias</span>
      <span class="font-semibold">{(p.brakeBiasFront * 100).toFixed(1)}%</span>
    </div>
    {#if p.electricMotorState > 0}
      <div class="mt-2 border-t border-hud-line pt-2">
        <div class="flex justify-between">
          <span class="text-muted">Hybrid</span>
          <span class="font-semibold">{MOTOR[p.electricMotorState] ?? "–"}</span>
        </div>
        <div class="mt-1 h-1.5 rounded-full bg-hud-2">
          <div class="h-full rounded-full bg-good" style:width="{p.battery * 100}%"></div>
        </div>
        <div class="mt-0.5 flex justify-between text-[11px] text-muted">
          <span>Battery {Math.round(p.battery * 100)}%</span>
          <span>Regen {Math.round(p.regenKw)} kW</span>
        </div>
      </div>
    {/if}
  {:else}
    <div class="py-3 text-center text-muted">No car</div>
  {/if}
</div>
