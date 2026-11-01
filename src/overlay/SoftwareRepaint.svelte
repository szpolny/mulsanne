<script lang="ts">
  // WebKitGTK 2.54's non-composited transparent backing store loses unchanged
  // pixels on partial paints. A transparent canvas invalidates the entire view
  // without tinting the game or changing widget state. GTK queue_draw alone
  // does not invalidate WebKit's backing store.
  let { active }: { active: boolean } = $props();
  const needed = (window as Window & { __MULSANNE_FULL_REPAINT__?: boolean }).__MULSANNE_FULL_REPAINT__;
  let canvas = $state<HTMLCanvasElement>();

  $effect(() => {
    if (!needed || !active || !canvas) return;
    const target = canvas;
    const context = target.getContext("2d");
    if (!context) return;
    let frame = 0;
    const paint = () => {
      if (target.width !== innerWidth) target.width = innerWidth;
      if (target.height !== innerHeight) target.height = innerHeight;
      context.clearRect(0, 0, target.width, target.height);
      frame = requestAnimationFrame(paint);
    };
    paint();
    return () => cancelAnimationFrame(frame);
  });
</script>

{#if needed && active}
  <canvas bind:this={canvas} class="pointer-events-none fixed inset-0 -z-10" aria-hidden="true"></canvas>
{/if}
