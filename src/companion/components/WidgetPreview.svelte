<script lang="ts">
  import { api } from "../../lib/api";
  import { config, type WidgetMeta } from "../../lib/config.svelte";
  import { live, provideLive } from "../../lib/live.svelte";
  import type { Snapshot } from "../../lib/types";
  import { COMPONENTS } from "../../overlay/widgets";

  let { meta }: { meta: WidgetMeta } = $props();

  const cfg = $derived(config.value.widgets[meta.id]);
  const g = $derived(config.value.general);
  const Widget = $derived(COMPONENTS[meta.id]);

  // Real telemetry while driving; otherwise preview-only demo data that never reaches the overlay.
  const usingDemo = $derived(!live.snap?.player);
  let demoSnap = $state.raw<Snapshot | null>(null);
  provideLive({
    get snap() {
      return usingDemo ? demoSnap : live.snap;
    },
  });

  $effect(() => {
    if (!usingDemo) return;
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      const snap = await api.getPreviewSnapshot().catch(() => null);
      if (stopped) return;
      if (snap) demoSnap = snap;
      timer = setTimeout(poll, 50);
    }
    poll();
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  });

  // Widget's unscaled size, measured so the scaled box takes up the right amount of layout space.
  let naturalW = $state(0);
  let naturalH = $state(0);
  let stageW = $state(0);

  const PAD = 24;
  const MAX_H = 360;
  const size = $derived(g.scale * cfg.size);
  const fit = $derived(
    naturalW && naturalH ? Math.min(1, (stageW - PAD * 2) / (naturalW * size), MAX_H / (naturalH * size)) : 1,
  );
  const scale = $derived(size * fit);
</script>

<div
  class="relative flex items-center justify-center overflow-hidden rounded-lg border border-line"
  style:min-height="160px"
  style:padding="{PAD + 8}px {PAD}px {PAD}px"
  style:background-image="linear-gradient(rgb(255 255 255 / 0.035) 1px, transparent 1px), linear-gradient(90deg, rgb(255 255 255 / 0.035) 1px, transparent 1px), radial-gradient(ellipse at 30% 20%, #3a4250 0%, #1c2028 55%, #101216 100%)"
  style:background-size="20px 20px, 20px 20px, 100% 100%"
  bind:clientWidth={stageW}
>
  <div style:width="{naturalW * scale}px" style:height="{naturalH * scale}px">
    <div
      class="pointer-events-none w-max origin-top-left"
      style:transform="scale({scale})"
      style:opacity={g.opacity * cfg.alpha}
      bind:clientWidth={naturalW}
      bind:clientHeight={naturalH}
    >
      {#key meta.id}
        <Widget opts={cfg.options} units={config.value.general.units} edit={false} />
      {/key}
    </div>
  </div>

  <div class="absolute top-2 left-3 flex items-center gap-2 text-[11px] font-semibold tracking-wider text-white/50 uppercase">
    Preview
    <span class={["rounded px-1.5 py-px", usingDemo ? "bg-warn/20 text-warn" : "bg-good/20 text-good"]}>
      {usingDemo ? "Demo data" : "Live"}
    </span>
  </div>
  {#if fit < 1}
    <div class="absolute top-2 right-3 text-[11px] text-white/50 tnum">
      Shown at {Math.round(scale * 100)}% to fit
    </div>
  {/if}
</div>
