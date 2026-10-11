/**
 * LMU world space is left-handed with +y up, so seen from above +x is right
 * and +z is up the screen. Screen coordinates are therefore (x, -z).
 */
export function toScreen(x: number, z: number): [number, number] {
  return [x, -z];
}

export interface Fit {
  scale: number;
  ox: number;
  oy: number;
}

/** Fits world points into a `w`×`h` box with `pad` margin, keeping aspect ratio. */
export function fit(points: [number, number][], w: number, h: number, pad = 8): Fit {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const [x, z] of points) {
    const [sx, sy] = toScreen(x, z);
    if (sx < minX) minX = sx;
    if (sx > maxX) maxX = sx;
    if (sy < minY) minY = sy;
    if (sy > maxY) maxY = sy;
  }
  if (!isFinite(minX)) return { scale: 1, ox: 0, oy: 0 };
  const scale = Math.min((w - pad * 2) / Math.max(1, maxX - minX), (h - pad * 2) / Math.max(1, maxY - minY));
  return {
    scale,
    ox: (w - (maxX - minX) * scale) / 2 - minX * scale,
    oy: (h - (maxY - minY) * scale) / 2 - minY * scale,
  };
}

export function project(f: Fit, x: number, z: number): [number, number] {
  const [sx, sy] = toScreen(x, z);
  return [sx * f.scale + f.ox, sy * f.scale + f.oy];
}

export function pathFrom(f: Fit, points: [number, number][], close = true): string {
  if (!points.length) return "";
  let d = "";
  for (let i = 0; i < points.length; i++) {
    const [px, py] = project(f, points[i][0], points[i][1]);
    d += `${i ? "L" : "M"}${px.toFixed(1)},${py.toFixed(1)}`;
  }
  return close ? d + "Z" : d;
}
