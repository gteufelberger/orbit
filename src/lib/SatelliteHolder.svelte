<script lang="ts">
  import type { Satellite } from "$bindings/Satellite";
  import { sim } from "$lib/simulation.svelte";
  import TelemetryPlot from "./TelemetryPlot.svelte";

  let { satellite }: { satellite: Satellite } = $props();

  let result = $derived(
    sim.result?.satellites.find((entry) => entry.id === satellite.id),
  );
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
    border-bottom: 1px solid #eee;
  }

  h2 {
    margin: 0;
    font-size: 0.95em;
  }

  .placeholder {
    margin: 0.3em 0 0;
    font-size: 0.75em;
    color: #999;
  }
</style>
