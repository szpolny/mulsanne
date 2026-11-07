import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, symlinkSync, readFileSync, existsSync, rmSync } from 'node:fs';
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
test('portable archive installs from paths with spaces and uninstalls without removing laps', {skip: !linux}, () => {
  const dir = mkdtempSync(join(tmpdir(),'mulsanne-install-'));
  try {
    const input=join(dir,'release files');mkdirSync(input);
    const app=join(input,'app.AppImage'),bridge=join(input,'lmu-bridge.exe'),archive=join(input,'release.tar.gz');
    writeFileSync(app,'#!/bin/sh\nexit 0\n');writeFileSync(bridge,'fixture');
    bash('bash "$ROOT/scripts/package-linux.sh" "$APP" "$BRIDGE" "$ARCHIVE"; tar -xzf "$ARCHIVE" -C "$INPUT"', {ROOT:root,APP:app,BRIDGE:bridge,ARCHIVE:archive,INPUT:input});
    const data=join(dir,'user data'), installer=join(input,'mulsanne-linux-experimental/install.sh');
    bash('bash "$INSTALLER"', {INSTALLER:installer,XDG_DATA_HOME:data});
    const desktop=readFileSync(join(data,'applications/xyz.mulsanne.linux.desktop'),'utf8');
    assert.match(desktop,/Experimental Linux/);
    assert.match(desktop,/Exec=bash ".*user data\/mulsanne-linux\/launch.sh"/);
    assert.ok(existsSync(join(data,'mulsanne-linux/lmu-bridge.exe')));
    mkdirSync(join(data,'xyz.mulsanne'));writeFileSync(join(data,'xyz.mulsanne/laps.db'),'laps');
    bash('bash "$INSTALLER" --uninstall', {INSTALLER:installer,XDG_DATA_HOME:data});
    assert.ok(!existsSync(join(data,'mulsanne-linux')));
    assert.equal(readFileSync(join(data,'xyz.mulsanne/laps.db'),'utf8'),'laps');
  } finally { rmSync(dir,{recursive:true,force:true}); }
});
test('Hyprland HUD policy handles fullscreen, other monitors, covered game and workspace switches', {skip:!linux}, t => {
  const version=spawnSync('lua',['-v']);
  if(version.error?.code==='ENOENT') return t.skip('Lua is not installed');
  const result=spawnSync('lua',['tests/hyprland.lua'],{cwd:root,encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);
});
