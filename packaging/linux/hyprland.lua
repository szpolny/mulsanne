if _G.mulsanne_linux_timer then _G.mulsanne_linux_timer:set_enabled(false) end
_G.mulsanne_linux_rule = hl.window_rule({
  name = "mulsanne-linux-hud",
  match = { class = "^Lm-overlay$", title = "^Mulsanne HUD$" },
  float = true,
  no_blur = true,
  no_dim = true,
  no_follow_mouse = true,
  no_shadow = true,
  no_anim = true,
  decorate = false,
  border_size = 0,
  rounding = 0,
})
local function coords(v)
  return v.x or v[1], v.y or v[2]
end
_G.mulsanne_linux_timer = hl.timer(function()
  local hud = hl.get_window("title:^Mulsanne HUD$")
  if not hud then return end
  local active = hl.get_active_window()
  local game = hl.get_window("class:^steam_app_2399420$")
  if not game then
    -- Keep layout editing available before the game starts.
    if hud.pinned then hl.dispatch(hl.dsp.window.pin({window=hud, action="unset"})) end
    local companion = hl.get_window("title:^Mulsanne$")
    if companion and companion.workspace and
      (not hud.workspace or hud.workspace.name ~= companion.workspace.name) then
      hl.dispatch(hl.dsp.window.move({window=hud, workspace=tostring(companion.workspace.id), follow=false}))
    end
    return
  end
  local monitor = game and game.monitor
  local workspace = monitor and monitor.active_workspace
  local visible = game and workspace and game.workspace and
    workspace.name == game.workspace.name
  local covered = active and game and active.monitor and monitor and
    active.monitor.id == monitor.id and
    active.address ~= game.address and active.address ~= hud.address
  if visible and not covered then
    if not hud.workspace or hud.workspace.name ~= game.workspace.name then
      hl.dispatch(hl.dsp.window.pin({window=hud, action="unset"}))
      hl.dispatch(hl.dsp.window.move({window=hud, workspace=tostring(game.workspace.id), follow=false}))
    end
    local x,y = coords(game.at)
    local w,h = coords(game.size)
    local hx,hy = coords(hud.at)
    local hw,hh = coords(hud.size)
    if hw ~= w or hh ~= h then
      hl.dispatch(hl.dsp.window.resize({window=hud, x=w, y=h, relative=false}))
    end
    if hx ~= x or hy ~= y then
      hl.dispatch(hl.dsp.window.move({window=hud, x=x, y=y, relative=false}))
    end
    if not hud.pinned then
      hl.dispatch(hl.dsp.window.pin({window=hud, action="set"}))
      hl.dispatch(hl.dsp.window.alter_zorder({window=hud, mode="top"}))
    end
  else
    if hud.pinned then hl.dispatch(hl.dsp.window.pin({window=hud, action="unset"})) end
    if not hud.workspace or hud.workspace.name ~= "special:mulsanne-linux-hidden" then
      hl.dispatch(hl.dsp.window.move({window=hud, workspace="special:mulsanne-linux-hidden", follow=false}))
    end
  end
end, {timeout=100, type="repeat"})
_G.mulsanne_linux_timer:set_enabled(true)
