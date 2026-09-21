<script lang="ts">
  import type { Satellite } from "$lib/types";
  import SatelliteHolder from "./SatelliteHolder.svelte";

  let satellite_list: Satellite[] = [];
</script>

{#each satellite_list as satellite (satellite.id)}
  <SatelliteHolder {satellite} />
{/each}
