<script lang="ts">
  import { onMount } from "svelte";
  import * as Cesium from "cesium";
  import "cesium/Build/Cesium/Widgets/widgets.css";

  let container: HTMLDivElement;
  let viewer: Cesium.Viewer;

  onMount(() => {
    if (!container) return;

    // Initialize Cesium Viewer
    viewer = new Cesium.Viewer(container, {});

    return () => {
      // Cleanup on unmount
      if (viewer && !viewer.isDestroyed()) {
        viewer.destroy();
      }
    };
  });
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
