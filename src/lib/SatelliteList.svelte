<script lang="ts">
  import { sim } from "$lib/simulation.svelte";
  import SatelliteHolder from "./SatelliteHolder.svelte";
</script>

{#each sim.satellites as satellite, index (satellite.id)}
  <SatelliteHolder {satellite} result={sim.result?.satellites[index]} />
{/each}
