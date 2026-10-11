<script lang="ts">
  import { brandOf } from "../lib/brands";

  let { model, entry = "", size = 16 }: { model: string; entry?: string; size?: number } = $props();

  const brand = $derived(brandOf(model, entry));

  // Fit the logo inside a size×size box while keeping its aspect ratio, so the
  // SVG viewport matches the viewBox exactly (nothing outside the crop shows).
  const dims = $derived.by(() => {
    if (!brand?.logo) return { w: size, h: size };
    const [, , vw, vh] = brand.logo.viewBox.split(/\s+/).map(Number);
    const aspect = vw / vh;
    return aspect >= 1 ? { w: size, h: size / aspect } : { w: size * aspect, h: size };
  });
</script>

{#if brand?.mask}
  <span
    class="inline-block shrink-0 bg-current opacity-90"
    style:width="{size}px"
    style:height="{size}px"
    style:mask="url({brand.mask}) center / contain no-repeat"
    style:-webkit-mask="url({brand.mask}) center / contain no-repeat"
    role="img"
    aria-label={brand.name}
    title={brand.name}
  ></span>
{:else if brand?.logo}
  <span class="inline-flex shrink-0 items-center justify-center" style:width="{size}px" style:height="{size}px" title={brand.name}>
    <svg width={dims.w} height={dims.h} viewBox={brand.logo.viewBox} fill="currentColor" class="opacity-90" aria-label={brand.name}>
      {#each brand.logo.paths as p}
        <path d={p.d} fill={p.cutout ? "var(--logo-cutout, #0e0e11)" : "currentColor"} fill-rule={p.evenodd ? "evenodd" : undefined} />
      {/each}
    </svg>
  </span>
{:else if brand}
  <span
    class="inline-flex shrink-0 items-center justify-center rounded-[4px] bg-white/12 font-display leading-none font-bold text-white/85"
    style:height="{size}px"
    style:min-width="{size}px"
    style:font-size="{Math.round(size * 0.55)}px"
    style:padding="0 2px"
    title={brand.name}>{brand.mono}</span
  >
{:else}
  <span class="inline-block shrink-0" style:width="{size}px"></span>
{/if}
