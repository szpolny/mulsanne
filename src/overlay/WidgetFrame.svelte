<script lang="ts">
  import type { Snippet } from "svelte";
  import { config, saveConfig, type WidgetConfig } from "../lib/config.svelte";

  let { id, name, edit, children }: { id: string; name: string; edit: boolean; children: Snippet } = $props();

  const cfg: WidgetConfig = $derived(config.value.widgets[id]);
  const g = $derived(config.value.general);

  let dragging = $state(false);
  let start = { px: 0, py: 0, x: 0, y: 0 };

  function snap(v: number) {
    const s = config.value.general.snap || 1;
    return Math.round(v / s) * s;
  }

  function onPointerDown(e: PointerEvent) {
    if (!edit || e.button !== 0) return;
    dragging = true;
    start = { px: e.clientX, py: e.clientY, x: cfg.x, y: cfg.y };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    cfg.x = Math.max(0, snap(start.x + e.clientX - start.px));
    cfg.y = Math.max(0, snap(start.y + e.clientY - start.py));
  }

  function onPointerUp() {
    if (!dragging) return;
    dragging = false;
    saveConfig();
  }

  function onWheel(e: WheelEvent) {
    if (!edit) return;
    e.preventDefault();
    const step = e.deltaY < 0 ? 0.05 : -0.05;
    g.scale = Math.min(3, Math.max(0.4, Math.round((g.scale + step) * 100) / 100));
    saveConfig();
  }
</script>

<div
  class="absolute top-0 left-0 origin-top-left"
  class:cursor-move={edit}
  class:z-10={dragging}
  style:transform="translate({cfg.x}px, {cfg.y}px) scale({g.scale})"
  style:opacity={g.opacity}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
  role="presentation"
>
  {#if edit}
    <div
      class="pointer-events-none absolute -inset-1 rounded-[10px] border border-dashed"
      style:border-color={dragging ? "var(--color-accent)" : "rgb(255 255 255 / 0.45)"}
    ></div>
    <div
      class="pointer-events-none absolute -top-6 left-0 flex items-center gap-2 rounded-md bg-accent px-2 py-0.5 text-[12px] font-semibold whitespace-nowrap text-white"
    >
      {name}
      <span class="font-normal opacity-80">{Math.round(g.scale * 100)}%</span>
    </div>
  {/if}
  <div class:pointer-events-none={edit}>
    {@render children()}
  </div>
</div>
