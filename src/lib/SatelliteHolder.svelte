<script lang="ts">
  import type { SatelliteResult } from "$bindings/SatelliteResult";
  import { sim, type DisplaySatellite } from "$lib/simulation.svelte";
  import TelemetryPlot from "./TelemetryPlot.svelte";

  let {
    satellite,
    result,
  }: { satellite: DisplaySatellite; result?: SatelliteResult } = $props();
</script>

<section>
  <h2>{satellite.name}</h2>

  {#if result && sim.result}
    {#each result.channels as channel (channel.spec.id)}
      <TelemetryPlot {channel} step_seconds={sim.result.step_seconds} />
    {/each}
  {:else}
    <p class="placeholder">
      {satellite.components.length} component{satellite.components.length === 1
        ? ""
        : "s"} · run a simulation to see telemetry
    </p>
  {/if}
</section>

<style>
  section {
    padding: 0.5em 0;
    border-bottom: 1px solid var(--line);
  }

  h2 {
    margin: 0;
    font-size: 0.95em;
  }

  .placeholder {
    margin: 0.3em 0 0;
    font-size: 0.75em;
    color: var(--fg-dim);
  }
</style>
