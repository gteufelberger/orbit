<script lang="ts">
  import { onMount } from "svelte";
  import * as Cesium from "cesium";
  import "cesium/Build/Cesium/Widgets/widgets.css";
  import { getWasm, type OrbitWasmModule } from "./wasm";

  let container: HTMLDivElement;
  let viewer: Cesium.Viewer;
  let lastClockTime: Cesium.JulianDate | null = null;
  let wasm: OrbitWasmModule;

  const EARTH_RADIUS = 6371000; // meters
  const ORBITAL_ALTITUDE = 400000; // meters (400 km)
  const ORBITAL_RADIUS = EARTH_RADIUS + ORBITAL_ALTITUDE;
  const ORBITAL_PERIOD = 92 * 60; // seconds (approximately 92 minutes for LEO)

  onMount(() => {
    if (!container) return;

    // Load WASM module and initialize viewer
    (async () => {
      wasm = await getWasm();

      // Initialize Cesium Viewer
      viewer = new Cesium.Viewer(container, {});

      // Create satellite entity with position callback
      viewer.entities.add({
        position: new Cesium.CallbackPositionProperty(
          () => {
            const currentTime = viewer.clock.currentTime;
            const seconds = Cesium.JulianDate.secondsDifference(
              currentTime,
              Cesium.JulianDate.fromDate(new Date(0)),
            );

            const position = wasm.calculate_satellite_position(
              seconds,
              ORBITAL_PERIOD,
              ORBITAL_RADIUS,
            );

            return new Cesium.Cartesian3(position[0], position[1], position[2]);
          },
          false,
          Cesium.ReferenceFrame.FIXED,
        ),
        point: {
          pixelSize: 10,
          color: Cesium.Color.RED,
          outlineColor: Cesium.Color.WHITE,
          outlineWidth: 2,
        },
      });

      // Create orbit path polyline
      const orbitPoints: Cesium.Cartesian3[] = [];
      const NUM_POINTS = 360;
      for (let i = 0; i <= NUM_POINTS; i++) {
        const angle = (i / NUM_POINTS) * 2 * Math.PI;
        orbitPoints.push(
          new Cesium.Cartesian3(
            ORBITAL_RADIUS * Math.cos(angle),
            ORBITAL_RADIUS * Math.sin(angle),
            0,
          ),
        );
      }
      viewer.entities.add({
        polyline: {
          positions: orbitPoints,
          width: 2,
          material: Cesium.Color.YELLOW.withAlpha(0.7),
          arcType: Cesium.ArcType.NONE,
        },
      });

      // Animate on clock changes
      viewer.clock.onTick.addEventListener(() => {
        const currentTime = viewer.clock.currentTime;
        if (
          !lastClockTime ||
          !Cesium.JulianDate.equals(lastClockTime, currentTime)
        ) {
          animate(currentTime);
          lastClockTime = Cesium.JulianDate.clone(currentTime);
        }
      });
    })();

    return () => {
      // Cleanup on unmount
      if (viewer && !viewer.isDestroyed()) {
        viewer.destroy();
      }
    };
  });

  function animate(currentTime: Cesium.JulianDate) {
    console.log(currentTime);
  }
</script>

<div bind:this={container} class="cesium-container"></div>

<style>
  .cesium-container {
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
