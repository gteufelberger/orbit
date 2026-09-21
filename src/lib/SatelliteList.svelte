<script lang="ts">
  import type { Satellite } from "$lib/types";
  import SatelliteHolder from "./SatelliteHolder.svelte";

  let satellite_list: Satellite[] = [
    {
      name: "Landsat 9",
      id: "landsat9",
      tle: "1 49260U 21088A   26264.47165275  .00000170  00000-0  47773-4 0  9997\n2 49260  98.2196 333.4388 0001506  93.9087 266.2284 14.57100580265024",
    },
    {
      name: "FLEX",
      id: "flex",
      tle: "1 A0690U 26209B   26264.59238340  .00000241  00000-0  12522-3 0  9990\n2 A0690  98.6262 330.6436 0001976  88.7260 271.4146 14.23497854   911",
    },
  ];
</script>

{#each satellite_list as satellite (satellite.id)}
  <SatelliteHolder {satellite} />
{/each}
