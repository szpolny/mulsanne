<script lang="ts">
  import type { Snippet } from "svelte";
  import CarRender from "./CarRender.svelte";
  import TrackOutline from "./TrackOutline.svelte";

  /** Header band: the car on a slanted panel, with the circuit outline beside it. */
  let { track, file, alt = "", children }: { track: string; file: string | null | undefined; alt?: string; children: Snippet } = $props();
</script>

<div class="relative flex min-h-[156px] items-center overflow-hidden">
  <div class="absolute inset-y-0 right-0 w-[62%] bg-gradient-to-r from-accent/30 to-accent/5 [clip-path:polygon(16%_0,100%_0,100%_100%,0_100%)]"></div>
  <TrackOutline {track} hideMissing class="absolute top-[6%] right-5 h-[88%] w-[14%] text-fg/45" />
  <div class="relative z-10 min-w-[250px] flex-1 p-5">{@render children()}</div>
  <CarRender {file} view="front" {alt} class="relative z-10 -my-5 mr-[17%] h-[176px] w-[420px] min-w-0 shrink object-contain drop-shadow-[0_10px_16px_rgb(0_0_0/0.5)]" />
</div>
