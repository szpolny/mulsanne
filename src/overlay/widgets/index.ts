import type { Component } from "svelte";
import type { Units } from "../../lib/config.svelte";
import Damage from "./Damage.svelte";
import Dashboard from "./Dashboard.svelte";
import Delta from "./Delta.svelte";
import Electronics from "./Electronics.svelte";
import Flags from "./Flags.svelte";
import Fuel from "./Fuel.svelte";
import Inputs from "./Inputs.svelte";
import LapTiming from "./LapTiming.svelte";
import Relative from "./Relative.svelte";
import SessionInfo from "./SessionInfo.svelte";
import Standings from "./Standings.svelte";
import TrackMap from "./TrackMap.svelte";
import Tyres from "./Tyres.svelte";
import Weather from "./Weather.svelte";

export interface WidgetProps {
  opts: Record<string, any>;
  units: Units;
  edit: boolean;
}

/** Widget id (see WIDGETS in config.svelte.ts) -> component. */
export const COMPONENTS: Record<string, Component<WidgetProps>> = {
  dashboard: Dashboard,
  inputs: Inputs,
  delta: Delta,
  laptiming: LapTiming,
  standings: Standings,
  relative: Relative,
  fuel: Fuel,
  tyres: Tyres,
  session: SessionInfo,
  flags: Flags,
  trackmap: TrackMap,
  weather: Weather,
  damage: Damage,
  electronics: Electronics,
};
