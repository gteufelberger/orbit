<script lang="ts">
  import { sim } from "$lib/simulation.svelte";
  import SatelliteHolder from "./SatelliteHolder.svelte";
</script>

{#each sim.satellites as satellite (satellite.id)}
  <SatelliteHolder {satellite} />
{/each}
