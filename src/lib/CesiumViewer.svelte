<script lang="ts">
  import { onMount } from "svelte";
  import * as Cesium from "cesium";
  import "cesium/Build/Cesium/Widgets/widgets.css";

  let container: HTMLDivElement;
  let viewer: Cesium.Viewer;
  let lastClockTime: Cesium.JulianDate | null = null;

  onMount(() => {
    if (!container) return;

    // Initialize Cesium Viewer
    viewer = new Cesium.Viewer(container, {});

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
