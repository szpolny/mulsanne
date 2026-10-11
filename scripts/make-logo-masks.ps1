# Turns raster logos into single-colour alpha masks (white + alpha), cropped
# to the emblem and scaled to fit 128 px. The overlay tints them via CSS mask.
#
# Usage: powershell -File scripts/make-logo-masks.ps1 -Source <dir with input images> -Out src/lib/logos/masks
param(
  [Parameter(Mandatory = $true)][string]$Source,
  [Parameter(Mandatory = $true)][string]$Out
)
Add-Type -AssemblyName System.Drawing

# How to decide which pixels belong to the emblem:
#   light  - keep bright pixels (white logo on a dark background)
#   dark   - keep pixels with a low minimum channel (coloured/dark logo on white); white becomes a cut-out
#   lum    - keep only very dark pixels by luminance (drops mid tones such as orange fills)
$logos = @(
  @{ name = "ligier";   file = "1.png"; mode = "light" },
  @{ name = "alpine";   file = "2.png"; mode = "dark" },
  @{ name = "duqueine"; file = "3.png"; mode = "dark" },
  @{ name = "ginetta";  file = "4.jpg"; mode = "lum" }
)

function Clamp01([double]$v) { if ($v -lt 0) { 0 } elseif ($v -gt 1) { 1 } else { $v } }

New-Item -ItemType Directory -Force $Out | Out-Null
foreach ($l in $logos) {
  $src = New-Object System.Drawing.Bitmap (Join-Path $Source $l.file)
  $w = $src.Width; $h = $src.Height
  $bmp = New-Object System.Drawing.Bitmap $w, $h, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp); $g.DrawImage($src, 0, 0, $w, $h); $g.Dispose(); $src.Dispose()

  $rect = New-Object System.Drawing.Rectangle 0, 0, $w, $h
  $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadWrite, $bmp.PixelFormat)
  $bytes = New-Object byte[] ($data.Stride * $h)
  [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)

  $minX = $w; $minY = $h; $maxX = -1; $maxY = -1
  for ($y = 0; $y -lt $h; $y++) {
    $row = $y * $data.Stride
    for ($x = 0; $x -lt $w; $x++) {
      $i = $row + $x * 4
      $b = $bytes[$i]; $gg = $bytes[$i + 1]; $r = $bytes[$i + 2]; $a = $bytes[$i + 3] / 255.0
      $mn = [Math]::Min($r, [Math]::Min($gg, $b))
      switch ($l.mode) {
        "light" { $f = Clamp01 (($mn - 140) / 80.0) }
        "dark"  { $f = 1 - (Clamp01 (($mn - 140) / 80.0)) }
        "lum"   { $lum = 0.2126 * $r + 0.7152 * $gg + 0.0722 * $b; $f = 1 - (Clamp01 (($lum - 60) / 60.0)) }
      }
      $alpha = [int][Math]::Round(255 * $f * $a)
      $bytes[$i] = 255; $bytes[$i + 1] = 255; $bytes[$i + 2] = 255; $bytes[$i + 3] = [byte]$alpha
      if ($alpha -gt 128) {
        if ($x -lt $minX) { $minX = $x }; if ($x -gt $maxX) { $maxX = $x }
        if ($y -lt $minY) { $minY = $y }; if ($y -gt $maxY) { $maxY = $y }
      }
    }
  }
  [System.Runtime.InteropServices.Marshal]::Copy($bytes, 0, $data.Scan0, $bytes.Length)
  $bmp.UnlockBits($data)

  $cw = $maxX - $minX + 1; $ch = $maxY - $minY + 1
  $scale = 128.0 / [Math]::Max($cw, $ch)
  $ow = [int][Math]::Max(1, [Math]::Round($cw * $scale)); $oh = [int][Math]::Max(1, [Math]::Round($ch * $scale))
  $outBmp = New-Object System.Drawing.Bitmap $ow, $oh, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($outBmp)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.DrawImage($bmp, (New-Object System.Drawing.Rectangle 0, 0, $ow, $oh), (New-Object System.Drawing.Rectangle $minX, $minY, $cw, $ch), [System.Drawing.GraphicsUnit]::Pixel)
  $g.Dispose(); $bmp.Dispose()
  $path = Join-Path $Out "$($l.name).png"
  $outBmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $outBmp.Dispose()
  "{0}: crop {1}x{2} at ({3},{4}) -> {5}x{6}  {7} bytes" -f $l.name, $cw, $ch, $minX, $minY, $ow, $oh, (Get-Item $path).Length
}
