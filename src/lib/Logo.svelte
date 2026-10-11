<script lang="ts" module>
  /** The brand mark: three slanted livery stripes. Same geometry as the app icon. */
  export const MARK_PATHS = [
    { d: "M6 52 L22 12 H32 L16 52 Z", fill: "#ffffff" },
    { d: "M20 52 L36 12 H46 L30 52 Z", fill: "#e3101f" },
    { d: "M34 52 L50 12 H60 L44 52 Z", fill: "#e3101f", opacity: 0.55 },
  ];
</script>

<script lang="ts">
  let { size = 28, wordmark = true, tagline = false }: { size?: number; wordmark?: boolean; tagline?: boolean } = $props();
</script>

<div class="flex items-center gap-2.5 select-none">
  <svg width={size * 1.1} height={size} viewBox="4 10 58 44" aria-hidden="true">
    {#each MARK_PATHS as p}
      <path d={p.d} fill={p.fill} opacity={p.opacity ?? 1} />
    {/each}
  </svg>
  {#if wordmark}
    <div class="leading-none">
      <div
        class="font-display font-extrabold tracking-[0.04em] uppercase italic"
        style:font-size="{size * 0.86}px"
      >
        Mulsanne
      </div>
      {#if tagline}
        <div class="mt-0.5 font-display text-[10px] font-semibold tracking-[0.32em] text-muted uppercase">
          LMU Telemetry
        </div>
      {/if}
    </div>
  {/if}
</div>
