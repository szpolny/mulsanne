<script lang="ts" module>
  export type ColorBy = "speed" | "inputs" | "gear" | "delta";
</script>

<script lang="ts">
  import type { Aligned, BrakeZone } from "../../lib/analysis";
  import { fit, project, type Fit } from "../../lib/geometry";

  let {
    data,
    colorBy,
    zones,
    cursorIdx,
    oncursor,
    height = 460,
  }: {
    data: Aligned;
    colorBy: ColorBy;
    zones: BrakeZone[];
    cursorIdx: number | null;
    oncursor: (idx: number | null) => void;
    height?: number;
  } = $props();

  let canvas: HTMLCanvasElement;
  let width = $state(400);
  let view = $state({ k: 1, tx: 0, ty: 0 });
  let drag = $state<{ x: number; y: number; tx: number; ty: number; moved: boolean } | null>(null);

  const points = $derived.by(() => {
    const pts: [number, number][] = [];
    for (let i = 0; i < data.d.length; i++) pts.push([data.a.x[i], data.a.z[i]]);
    return pts;
  });
  const base: Fit = $derived(fit(points, width, height, 24));

  // Reset zoom when a new lap is loaded.
  $effect(() => {
    void data;
    view = { k: 1, tx: 0, ty: 0 };
  });

  function toScreen(x: number, z: number): [number, number] {
    const [px, py] = project(base, x, z);
    return [px * view.k + view.tx, py * view.k + view.ty];
  }

  const GEAR_COLORS = ["#8b93a7", "#e5484d", "#f2a33a", "#f5d000", "#2fbf71", "#3ec7c7", "#3e8ef7", "#a15ef0", "#e85ab4"];

  function speedColor(v: number, lo: number, hi: number) {
    const f = Math.max(0, Math.min(1, (v - lo) / Math.max(1, hi - lo)));
    return `hsl(${(1 - f) * 240} 85% 55%)`;
  }

  function segmentColor(i: number, lo: number, hi: number): string {
    const a = data.a;
    switch (colorBy) {
      case "speed":
        return speedColor(a.speed[i], lo, hi);
      case "inputs":
        if (a.brake[i] > 0.05) return `rgb(229 72 77 / ${0.35 + a.brake[i] * 0.65})`;
        if (a.throttle[i] > 0.05) return `rgb(47 191 113 / ${0.25 + a.throttle[i] * 0.75})`;
        return "#8b93a7";
      case "gear":
        return GEAR_COLORS[Math.max(0, Math.min(8, Math.round(a.gear[i])))];
      case "delta": {
        if (!data.delta) return "#8b93a7";
        const j = Math.min(data.delta.length - 1, i + 5);
        const k = Math.max(0, i - 5);
        const rate = (data.delta[j] - data.delta[k]) / Math.max(1, data.d[j] - data.d[k]);
        // Seconds lost per metre: red when A is losing time, green when gaining.
        const f = Math.max(-1, Math.min(1, rate * 400));
        return f > 0 ? `rgb(229 72 77 / ${0.3 + f * 0.7})` : `rgb(47 191 113 / ${0.3 - f * 0.7})`;
      }
    }
  }

  function draw() {
    const ctx = canvas?.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    const n = data.d.length;
    const lw = Math.min(8, 3 * Math.sqrt(view.k));

    // Reference lap (B) as a wide faint ghost line.
    if (data.b) {
      ctx.strokeStyle = "rgb(56 189 248 / 0.4)";
      ctx.lineWidth = lw + 3;
      ctx.beginPath();
      for (let i = 0; i < n; i++) {
        const [x, y] = toScreen(data.b.x[i], data.b.z[i]);
        if (i) ctx.lineTo(x, y);
        else ctx.moveTo(x, y);
      }
      ctx.stroke();
    }

    let lo = Infinity, hi = -Infinity;
    for (let i = 0; i < n; i++) {
      lo = Math.min(lo, data.a.speed[i]);
      hi = Math.max(hi, data.a.speed[i]);
    }

    ctx.lineWidth = lw;
    let prev = toScreen(data.a.x[0], data.a.z[0]);
    for (let i = 1; i < n; i++) {
      const cur = toScreen(data.a.x[i], data.a.z[i]);
      ctx.strokeStyle = segmentColor(i, lo, hi);
      ctx.beginPath();
      ctx.moveTo(prev[0], prev[1]);
      ctx.lineTo(cur[0], cur[1]);
      ctx.stroke();
      prev = cur;
    }

    // Start/finish
    const [sx, sy] = toScreen(data.a.x[0], data.a.z[0]);
    ctx.fillStyle = "#fff";
    ctx.fillRect(sx - 4, sy - 4, 8, 8);

    // Braking zone numbers
    ctx.font = "600 10px Segoe UI";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    zones.forEach((z, k) => {
      const [x, y] = toScreen(data.a.x[z.start], data.a.z[z.start]);
      ctx.fillStyle = "rgb(229 72 77 / 0.9)";
      ctx.beginPath();
      ctx.arc(x, y, 8, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = "#fff";
      ctx.fillText(String(k + 1), x, y + 0.5);
    });

    if (cursorIdx != null && cursorIdx < n) {
      if (data.b) {
        const [bx, by] = toScreen(data.b.x[cursorIdx], data.b.z[cursorIdx]);
        ctx.fillStyle = "#38bdf8";
        ctx.beginPath();
        ctx.arc(bx, by, 5.5, 0, Math.PI * 2);
        ctx.fill();
      }
      const [ax, ay] = toScreen(data.a.x[cursorIdx], data.a.z[cursorIdx]);
      ctx.fillStyle = "#fff";
      ctx.strokeStyle = "#e3101f";
      ctx.lineWidth = 2.5;
      ctx.beginPath();
      ctx.arc(ax, ay, 6.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    }
  }

  $effect(() => {
    void [data, colorBy, zones, cursorIdx, width, height, view.k, view.tx, view.ty, base];
    draw();
  });

  function nearest(mx: number, my: number): number | null {
    let best = -1;
    let bestD = 18 * 18;
    for (let i = 0; i < data.d.length; i += 2) {
      const [x, y] = toScreen(data.a.x[i], data.a.z[i]);
      const dd = (x - mx) ** 2 + (y - my) ** 2;
      if (dd < bestD) {
        bestD = dd;
        best = i;
      }
    }
    return best >= 0 ? best : null;
  }

  function onMove(e: PointerEvent) {
    const r = canvas.getBoundingClientRect();
    const mx = e.clientX - r.left;
    const my = e.clientY - r.top;
    if (drag) {
      const dx = e.clientX - drag.x;
      const dy = e.clientY - drag.y;
      if (Math.abs(dx) + Math.abs(dy) > 3) drag.moved = true;
      view = { ...view, tx: drag.tx + dx, ty: drag.ty + dy };
      return;
    }
    const idx = nearest(mx, my);
    if (idx != null) oncursor(idx);
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const r = canvas.getBoundingClientRect();
    const mx = e.clientX - r.left;
    const my = e.clientY - r.top;
    const factor = e.deltaY < 0 ? 1.2 : 1 / 1.2;
    const k = Math.max(1, Math.min(30, view.k * factor));
    const f = k / view.k;
    view = { k, tx: mx - (mx - view.tx) * f, ty: my - (my - view.ty) * f };
  }
</script>

<div class="relative overflow-hidden card" bind:clientWidth={width}>
  <canvas
    bind:this={canvas}
    style:width="{width}px"
    style:height="{height}px"
    class={["block", drag ? "cursor-grabbing" : "cursor-crosshair"]}
    onpointermove={onMove}
    onpointerdown={(e) => {
      drag = { x: e.clientX, y: e.clientY, tx: view.tx, ty: view.ty, moved: false };
      canvas.setPointerCapture(e.pointerId);
    }}
    onpointerup={() => (drag = null)}
    onwheel={onWheel}
    ondblclick={() => (view = { k: 1, tx: 0, ty: 0 })}
  ></canvas>
  <div class="pointer-events-none absolute bottom-2 left-3 text-[11px] text-faint">Scroll to zoom · drag to pan · double-click to reset</div>
</div>
