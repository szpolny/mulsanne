#!/usr/bin/env node
// Packaged UI verification on a private display, with isolated app data.
import assert from 'node:assert/strict';
import {spawn, spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, openSync, closeSync, writeFileSync, renameSync, existsSync, readdirSync, rmSync, copyFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve, join} from 'node:path';
import {randomInt} from 'node:crypto';
import {setTimeout as delay} from 'node:timers/promises';
const binary=resolve(process.argv[2] || 'src-tauri/target/release/lm-overlay');
const output=resolve(process.argv[3] || 'src-tauri/target/smoke-linux');
mkdirSync(output,{recursive:true});
const data=mkdtempSync(join(tmpdir(),'mulsanne-smoke-'));
const log=openSync(join(output,'app.log'),'w');
const env={...process.env, LANG:'C.UTF-8',LC_ALL:'C.UTF-8',LANGUAGE:'C',WEBKIT_DISABLE_DMABUF_RENDERER:'1',
  XDG_STATE_HOME:join(data,'state'), XDG_DATA_HOME:join(data,'data'), XDG_CONFIG_HOME:join(data,'config'),APPIMAGE_EXTRACT_AND_RUN:'1',MULSANNE_LMU_SHM:join(data,'LMU_Data')};
delete env.WAYLAND_DISPLAY;delete env.GDK_BACKEND;delete env.HYPRLAND_INSTANCE_SIGNATURE;
env.NO_AT_BRIDGE='1';
env.XDG_CURRENT_DESKTOP='Openbox';env.XDG_SESSION_TYPE='x11';
if (process.getuid()===0) env.WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS='1';
const children=[];
let writer;
function start(command,args=[]) {
  const child=spawn(command,args,{env,stdio:['ignore',log,log]});
  child.on('error',e=>{child.startError=e;});children.push(child);return child;
}
function result(command,args) {return spawnSync(command,args,{env,encoding:'utf8'});}
function run(command,...args) {const r=result(command,args);assert.equal(r.status,0,r.stderr||r.error?.message);return r.stdout.trim();}
let app;
async function waitFor(condition, timeout=20000) {
  const deadline=Date.now()+timeout;
  while (!condition()) {
    if(app) assert.ok(app.exitCode===null&&!app.startError,`App exited; see ${output}/app.log`);
    assert.ok(Date.now()<deadline,'UI smoke test timed out');await delay(100);
  }
}
function window(title) {const r=result('xdotool',['search','--name',`^${title}$`]);return r.status===0?r.stdout.trim().split('\n').at(-1):null;}
function geometry(id) {
  const text=run('xwininfo','-id',id,'-stats');
  return Object.fromEntries([['x','Absolute upper-left X'],['y','Absolute upper-left Y'],['w','Width'],['h','Height']].map(([key,label])=>[key,Number(text.match(new RegExp(`^\\s*${label}:\\s*(-?\\d+)`,'m'))[1])]));
}
try {
  const fixture=join(data,'x11-smoke');
  run('cc',resolve(import.meta.dirname,'x11-smoke.c'),'-o',fixture,'-lX11','-lXext');
  mkdirSync(join(env.XDG_CONFIG_HOME,'xyz.mulsanne'),{recursive:true});
  const names=['dashboard','inputs','delta','laptiming','standings','relative','fuel','tyres','session','flags','trackmap','weather','damage','electronics'];
  writeFileSync(join(env.XDG_CONFIG_HOME,'xyz.mulsanne/config.json'),JSON.stringify({general:{onlyOnTrack:false,showChangelog:false},widgets:Object.fromEntries(names.map(name=>[name,{enabled:name==='session',x:20,y:20}]))}));
  const frame=Buffer.alloc(324825);frame.writeInt32LE(1,1632+64);frame.writeDoubleLE(3600,1632+76);frame[128464]=frame[128466]=1;
  const begun=Date.now();
  function telemetry() {const elapsed=(Date.now()-begun)/1000;frame.writeDoubleLE(elapsed,1632+68);frame.writeDoubleLE(elapsed,128468+12);writeFileSync(env.MULSANNE_LMU_SHM+'.next',frame);renameSync(env.MULSANNE_LMU_SHM+'.next',env.MULSANNE_LMU_SHM);}
  telemetry();writer=setInterval(telemetry,1000/30);
  let number;
  do {number=100+randomInt(10000);} while ([`/tmp/.X${number}-lock`,`/tmp/.X11-unix/X${number}`,`/tmp/.X11-unix/X${number}_`].some(existsSync));
  env.DISPLAY=`:${number}`;
  const server=start('Xvfb',[env.DISPLAY,'-nolisten','tcp','-screen','0','1600x1000x24']);
  await waitFor(()=>{assert.ok(server.exitCode===null&&!server.startError,'Xvfb failed');return result('xdotool',['getdisplaygeometry']).status===0;});
  start('openbox');start('xcompmgr');await delay(1000);app=start(binary);
  await waitFor(()=>window('Mulsanne')&&window('Mulsanne HUD'),60000);
  const overlay=window('Mulsanne HUD');
  assert.match(run('xprop','-id',overlay,'_NET_WM_WINDOW_TYPE'),/_NET_WM_WINDOW_TYPE_DOCK/);
  await delay(3000);
  start(fixture);await waitFor(()=>window('Mulsanne smoke game'));
  const fake=window('Mulsanne smoke game');
  const sameBounds=()=>JSON.stringify(geometry(fake))===JSON.stringify(geometry(overlay));
  const area=()=>Number(run(fixture,'area',overlay));
  await waitFor(sameBounds);
  run('xdotool','windowmove',fake,'140','110');run('xdotool','windowsize',fake,'1000','800');
  await waitFor(sameBounds);await waitFor(()=>area()<=1);
  const hud=geometry(overlay),crop=`260x18+${hud.x+24}+${hud.y+24}`;
  for(let sample=0;sample<4;sample++) {
    await delay(300);const shot=join(output,`live-${sample}.png`);run('import','-window','root',shot);
    const pixels=Number(run('convert',shot,'-crop',crop,'-colorspace','Gray','-threshold','40%','-format','%[fx:mean*w*h]','info:').split('\n').at(-1));
    assert.ok(pixels>30,`Static HUD labels disappeared: ${pixels} pixels`);
  }
  run('import','-window','root',join(output,'initial.png'));run('xdotool','key','ctrl+shift+F9');
  await waitFor(()=>area()>1000);await delay(1000);run('import','-window','root',join(output,'edit.png'));
  const diff=result('compare',['-metric','AE',join(output,'initial.png'),join(output,'edit.png'),'null:']);
  assert.ok([0,1].includes(diff.status),diff.stderr);assert.ok(parseFloat(diff.stderr)>1000,'Edit UI did not render');
  run('xdotool','key','ctrl+shift+F9');await waitFor(()=>area()<=1);
  run('xdotool','key','ctrl+shift+F10');await waitFor(()=>run('xwininfo','-id',overlay,'-stats').includes('IsUnMapped'));
  run('xdotool','key','ctrl+shift+F10');await waitFor(()=>run('xwininfo','-id',overlay,'-stats').includes('IsViewable'));
  // Showing and resizing must preserve click-through outside edit mode.
  await waitFor(()=>area()<=1);
  assert.ok(readdirSync(data,{recursive:true}).some(p=>p.endsWith('laps.db')),'No lap database');
  const companion=window('Mulsanne');
  run('import','-window',companion,join(output,'companion.png'));
  console.log(`PASS: startup, live rendering, game bounds/resize, input regions, editing, shortcuts, visibility, data storage\nArtifacts: ${output}`);
} finally {
  clearInterval(writer);
  for(const child of children.reverse()) {
    if(child.exitCode!==null||child.startError) continue;
    const done=new Promise(r=>child.once('exit',r));child.kill('SIGTERM');
    await Promise.race([done,delay(2000)]);if(child.exitCode===null) {child.kill('SIGKILL');await done;}
  }
  for(const name of ['app.log','bridge.log']) {
    const path=join(env.XDG_STATE_HOME,'mulsanne',name);
    if(existsSync(path)) copyFileSync(path,join(output,`launcher-${name}`));
  }
  closeSync(log);rmSync(data,{recursive:true,force:true});
}
