import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, symlinkSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
const root = resolve(import.meta.dirname, '..');
const linux = process.platform === 'linux';
function bash(script, env = {}) {
  const result = spawnSync('bash', ['-c', script], { env: { ...process.env, ...env }, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
test('bridge selects game process and forwards only runtime variables', {skip: !linux}, () => {
  const dir = mkdtempSync(join(tmpdir(), 'mulsanne-proc-'));
  try {
    for (const [pid, command] of [[10, 'steam\0Le Mans Ultimate.exe\0'], [20, 'Z:\\Steam Library\\Le Mans Ultimate.exe\0']]) {
      mkdirSync(join(dir, String(pid), 'ns'), {recursive:true});
      writeFileSync(join(dir, String(pid), 'cmdline'), command);
      writeFileSync(join(dir, String(pid), 'environ'), 'WINEPREFIX=/Steam Library/prefix\0WINELOADER=/Proton/wine64\0LD_LIBRARY_PATH=/runtime/libs\0PRIVATE_TOKEN=secret\0');
      for (const ns of ['user','mnt']) symlinkSync(ns === 'mnt' ? 'different' : 'same', join(dir,String(pid),'ns',ns));
    }
    mkdirSync(join(dir,'self','ns'),{recursive:true});
    for (const ns of ['user','mnt']) symlinkSync('same',join(dir,'self','ns',ns));
    const output = bash('source "$LAUNCHER"; find_game "$FIXTURE"; printf "%s\\n" "$GAME_PID" "$GAME_WINE" "${GAME_RUNTIME[@]}" "${GAME_NAMESPACES[@]}"', {LAUNCHER:join(root,'scripts/run-lmu-bridge.sh'),FIXTURE:dir});
    assert.match(output,/^20\n\/Proton\/wine64\n/);
    assert.match(output,/WINEPREFIX=\/Steam Library\/prefix/);
    assert.match(output,/--mount/);
    assert.doesNotMatch(output,/PRIVATE_TOKEN|--user/);
    rmSync(join(dir,'20'),{recursive:true});
    bash('source "$LAUNCHER"; if find_game "$FIXTURE"; then exit 1; fi', {LAUNCHER:join(root,'scripts/run-lmu-bridge.sh'),FIXTURE:dir});
  } finally { rmSync(dir,{recursive:true,force:true}); }
});
test('Hyprland HUD policy handles fullscreen, other monitors, covered game and workspace switches', {skip:!linux}, t => {
  const version=spawnSync('lua',['-v']);
  if(version.error?.code==='ENOENT') return t.skip('Lua is not installed');
  const result=spawnSync('lua',['tests/hyprland.lua'],{cwd:root,encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);
});
test('native launcher starts packaged UI and bridge with the same mirror', {skip:!linux}, () => {
  const dir=mkdtempSync(join(tmpdir(),'mulsanne-package-'));
  try {
    // The UI stays alive long enough for one bridge attempt; no game is launched.
    writeFileSync(join(dir,'ui'), '#!/bin/sh\nprintf "%s" "$MULSANNE_LMU_SHM" > "$TEST_ROOT/ui-mirror"\nsleep 1\n');
    writeFileSync(join(dir,'run-lmu-bridge.sh'), '#!/bin/sh\nprintf "%s\\n" "$@" > "$TEST_ROOT/bridge-args"\n');
    bash('chmod +x "$TEST_ROOT/ui"; bash "$ROOT/packaging/linux/launch.sh"', {
      TEST_ROOT:dir,ROOT:root,MULSANNE_LIBEXEC:dir,MULSANNE_BINARY:join(dir,'ui'),
      MULSANNE_LMU_SHM:'/dev/shm/custom mirror',XDG_STATE_HOME:join(dir,'state'),HYPRLAND_INSTANCE_SIGNATURE:''
    });
    assert.equal(readFileSync(join(dir,'ui-mirror'),'utf8'),'/dev/shm/custom mirror');
    assert.equal(readFileSync(join(dir,'bridge-args'),'utf8'),`${dir}/lmu-bridge.exe\nZ:\\dev\\shm\\custom mirror\n`);
  } finally {rmSync(dir,{recursive:true,force:true});}
});
test('packaged bridge is cached outside the host /usr before entering Steam runtime', {skip:!linux}, () => {
  const dir=mkdtempSync(join(tmpdir(),'mulsanne-bridge-cache-'));
  try {
    const bridge=join(dir,'packaged.exe');writeFileSync(bridge,'bridge-v1');
    const env={LAUNCHER:join(root,'scripts/run-lmu-bridge.sh'),BRIDGE:bridge,XDG_CACHE_HOME:join(dir,'cache')};
    const first=bash('source "$LAUNCHER"; cache_bridge "$BRIDGE"',env).trim();
    assert.equal(readFileSync(first,'utf8'),'bridge-v1');
    assert.equal(bash('source "$LAUNCHER"; cache_bridge "$BRIDGE"',env).trim(),first);
    writeFileSync(bridge,'bridge-v2');
    const second=bash('source "$LAUNCHER"; cache_bridge "$BRIDGE"',env).trim();
    assert.notEqual(second,first);assert.equal(readFileSync(second,'utf8'),'bridge-v2');
    assert.equal(readFileSync(first,'utf8'),'bridge-v1');
  } finally {rmSync(dir,{recursive:true,force:true});}
});
