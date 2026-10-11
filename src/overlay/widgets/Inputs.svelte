<script lang="ts">
  import type { Units } from "../../lib/config.svelte";
  import { live } from "../../lib/live.svelte";

  let { opts }: { opts: Record<string, any>; units: Units; edit: boolean } = $props();

  const HEIGHT = 90;
  const width = $derived(Math.round(opts.width ?? 320));

  let canvas: HTMLCanvasElement | undefined = $state();
  // Ring buffer of [time, throttle, brake, steering].
  const samples: [number, number, number, number][] = [];
  let lastTime = NaN;

  $effect(() => {
    const snap = live.snap;
    const p = snap?.player;
    if (!snap || !p || p.elapsedTime === lastTime) return;
    lastTime = p.elapsedTime;
    const last = samples.at(-1);
    if (last && p.elapsedTime < last[0]) samples.length = 0; // session restart
    samples.push([p.elapsedTime, p.throttle, p.brake, p.steering]);
    const horizon = p.elapsedTime - (opts.seconds ?? 6);
    while (samples.length && samples[0][0] < horizon) samples.shift();
    draw(p.elapsedTime);
  });

  function draw(now: number) {
    const ctx = canvas?.getContext("2d");
    if (!ctx || !canvas) return;
    const dpr = window.devicePixelRatio || 1;
    if (canvas.width !== width * dpr) {
      canvas.width = width * dpr;
      canvas.height = HEIGHT * dpr;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, HEIGHT);

    const span = opts.seconds ?? 6;
    const xOf = (t: number) => width - ((now - t) / span) * width;
    const pad = 4;
    const yOf = (v: number) => HEIGHT - pad - v * (HEIGHT - pad * 2);

    ctx.strokeStyle = "rgb(255 255 255 / 0.07)";
    ctx.lineWidth = 1;
    for (const v of [0.25, 0.5, 0.75]) {
      ctx.beginPath();
      ctx.moveTo(0, yOf(v));
      ctx.lineTo(width, yOf(v));
      ctx.stroke();
    }

    const line = (idx: 1 | 2 | 3, color: string, map: (v: number) => number, w = 2) => {
      ctx.strokeStyle = color;
      ctx.lineWidth = w;
      ctx.lineJoin = "round";
      ctx.beginPath();
      samples.forEach((s, i) => {
        const x = xOf(s[0]);
        const y = yOf(map(s[idx]));
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      });
      ctx.stroke();
    };
    const css = getComputedStyle(document.documentElement);
    if (opts.showSteering) line(3, "rgb(245 245 245 / 0.65)", (v) => (v + 1) / 2, 1.5);
    line(1, css.getPropertyValue("--color-throttle"), (v) => v);
    line(2, css.getPropertyValue("--color-brake"), (v) => v);
  }
</script>

<div class="hud p-1.5">
  <canvas bind:this={canvas} style:width="{width}px" style:height="{HEIGHT}px" class="block"></canvas>
</div>
