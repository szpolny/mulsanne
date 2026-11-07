-- Exercise compositor policy without touching a user's compositor.
local game_monitor = {id=1, active_workspace={name="11"}}
local other_monitor = {id=0, active_workspace={name="4"}}
local game = {address="game", monitor=game_monitor, workspace={id=11,name="11"},at={x=1920,y=0},size={x=2560,y=1440}}
local hud = {address="hud", monitor=other_monitor, workspace={id=4,name="4"},at={x=0,y=360},size={x=1920,y=1080},pinned=false}
local active=game
local callback, rule
local actions={}
hl={dsp={window={}}}
for _,name in ipairs({"pin","move","resize","alter_zorder"}) do
  hl.dsp.window[name]=function(args) return {name=name,args=args} end
end
function hl.dispatch(command)
  local a=command.args
  actions[#actions+1]=command
  if command.name=="pin" then hud.pinned=a.action=="set"
  elseif command.name=="move" then
    if a.workspace then hud.workspace={id=tonumber(a.workspace),name=a.workspace};hud.monitor=game_monitor
    else hud.at={x=a.x,y=a.y} end
  elseif command.name=="resize" then hud.size={x=a.x,y=a.y} end
end
function hl.get_window(selector)
  if selector:find("Mulsanne HUD",1,true) then return hud end
  if selector=="title:^Mulsanne$" then return {workspace={id=11,name="11"}} end
  return game
end
function hl.get_active_window() return active end
function hl.window_rule(spec) rule=spec;return {} end
function hl.timer(cb,opts)
  assert(opts.type=="repeat")
  callback=cb
  return {set_enabled=function(_,enabled) assert(type(enabled)=="boolean") end}
end
dofile("packaging/linux/hyprland.lua")
assert(rule.no_blur and rule.no_dim and not rule.decorate and rule.border_size==0 and rule.rounding==0)
callback()
assert(hud.pinned and hud.workspace.name=="11")
assert(hud.at.x==1920 and hud.at.y==0 and hud.size.x==2560 and hud.size.y==1440)
active={address="other-screen-app",monitor=other_monitor}
callback()
assert(hud.pinned and hud.workspace.name=="11", "other-monitor focus must keep HUD visible")
active={address="covering-app",monitor=game_monitor}
callback()
assert(not hud.pinned and hud.workspace.name=="special:mulsanne-linux-hidden", "same-monitor apps must not be covered")
active=game;game_monitor.active_workspace={name="12"}
callback()
assert(not hud.pinned and hud.workspace.name=="special:mulsanne-linux-hidden", "HUD must not follow workspace switches")
game_monitor.active_workspace={name="11"};active=game
callback()
assert(hud.pinned and hud.workspace.name=="11")
active=hud;callback()
assert(hud.pinned, "layout editor must remain usable")
game=nil;callback()
assert(not hud.pinned and hud.workspace.name=="11", "layout editing must remain available without a game")
print("PASS: fullscreen geometry, decoration rules, other-monitor focus, workspace switches, covered game, edit focus")
