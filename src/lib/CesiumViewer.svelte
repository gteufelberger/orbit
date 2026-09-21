<script lang="ts">
  import { onMount } from "svelte";
  import * as Cesium from "cesium";
  import "cesium/Build/Cesium/Widgets/widgets.css";
  import { sim } from "$lib/simulation.svelte";
  import type { SatelliteResult } from "$bindings/SatelliteResult";
  import type { SimulationResult } from "$bindings/SimulationResult";

  let container: HTMLDivElement;
  let viewer: Cesium.Viewer | undefined = $state();

  const orbit_colors = [
    Cesium.Color.CYAN,
    Cesium.Color.ORANGE,
    Cesium.Color.LIME,
    Cesium.Color.MAGENTA,
  ];

  /**
   * How often the clock is published to shared state, in milliseconds.
   *
   * onTick fires every frame, but the plots only need enough resolution to move
   * the marker smoothly; republishing at 60 Hz would re-render them for
   * sub-pixel changes.
   */
  const clock_publish_interval_ms = 100;
  let last_published_ms = 0;

  onMount(() => {
    if (!container) return;

    // Initialize Cesium Viewer
    Cesium.Ion.defaultAccessToken =
      "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiIzMWZmY2E5Zi00OWQ0LTRhMjgtYTBkNS00MWQ5YTMyNTMyNTIiLCJpZCI6NDE5OTUzLCJpYXQiOjE3NzY1MjE2NTV9.TH0VeihQtFm8F0nwQoGJZSj3lKCUE9v9KAsL00mXPVI";
    viewer = new Cesium.Viewer(container, {});

    // The viewer owns the clock, so the current time only ever flows outwards:
    // the plots read it, nothing writes it back.
    const stop_listening = viewer.clock.onTick.addEventListener((clock) => {
      const now_ms = performance.now();
      if (now_ms - last_published_ms < clock_publish_interval_ms) return;

      last_published_ms = now_ms;
      publish_clock(clock.currentTime);
    });

    return () => {
      // Cleanup on unmount
      stop_listening();
      if (viewer && !viewer.isDestroyed()) {
        viewer.destroy();
      }
    };
  });

  function publish_clock(time: Cesium.JulianDate): void {
    sim.current_unix_seconds = Cesium.JulianDate.toDate(time).getTime() / 1000;
  }

  // Rebuild the scene whenever a new simulation lands. There is no per-tick
  // work: the sampled positions below let Cesium interpolate on its own clock.
  $effect(() => {
    const result = sim.result;
    if (!viewer || viewer.isDestroyed() || !result) return;

    show_result(viewer, result);
  });

  function show_result(viewer: Cesium.Viewer, result: SimulationResult): void {
    viewer.entities.removeAll();

    const start = Cesium.JulianDate.fromDate(
      new Date(result.start_unix_seconds * 1000),
    );
    const stop = Cesium.JulianDate.addSeconds(
      start,
      (result.sample_count - 1) * result.step_seconds,
      new Cesium.JulianDate(),
    );

    result.satellites.forEach((satellite, index) => {
      const color = orbit_colors[index % orbit_colors.length];
      const position = sampled_position(satellite, result, start);

      viewer.entities.add({
        id: satellite.id,
        name: satellite.name,
        position,
        orientation: new Cesium.VelocityOrientationProperty(position),
        point: {
          pixelSize: 10,
          color,
          outlineColor: Cesium.Color.BLACK,
          outlineWidth: 1,
        },
        label: {
          text: satellite.name,
          font: "12px sans-serif",
          pixelOffset: new Cesium.Cartesian2(0, -18),
          fillColor: color,
        },
        path: new Cesium.PathGraphics({
          width: 2,
          material: color,
          // Only the orbit already flown: one full orbit behind, nothing ahead.
          leadTime: 0,
          trailTime: 6000,
          resolution: result.step_seconds,
        }),
      });
    });

    viewer.clock.startTime = start.clone();
    viewer.clock.stopTime = stop.clone();
    viewer.clock.currentTime = start.clone();
    viewer.clock.clockRange = Cesium.ClockRange.LOOP_STOP;
    viewer.clock.multiplier = 60;
    viewer.clock.shouldAnimate = true;
    viewer.timeline?.zoomTo(start, stop);

    // Publish immediately so the plot markers snap to the new window's start
    // rather than waiting for the throttle window to elapse.
    publish_clock(viewer.clock.currentTime);
  }

  /**
   * Builds an interpolatable position track from the flat TEME samples.
   *
   * SGP4 produces TEME, which Cesium cannot use directly, so each sample is
   * rotated into the Earth-fixed frame with Cesium's own TEME transform.
   */
  function sampled_position(
    satellite: SatelliteResult,
    result: SimulationResult,
    start: Cesium.JulianDate,
  ): Cesium.SampledPositionProperty {
    const position = new Cesium.SampledPositionProperty();
    position.setInterpolationOptions({
      interpolationDegree: 5,
      interpolationAlgorithm: Cesium.LagrangePolynomialApproximation,
    });

    // The vectors are packed into numbers by addSample, so one scratch each is
    // enough. The JulianDate is kept by reference, so that one must be fresh.
    const teme = new Cesium.Cartesian3();
    const fixed = new Cesium.Cartesian3();
    const teme_to_fixed = new Cesium.Matrix3();

    for (let sample = 0; sample < result.sample_count; sample += 1) {
      const time = Cesium.JulianDate.addSeconds(
        start,
        sample * result.step_seconds,
        new Cesium.JulianDate(),
      );

      Cesium.Cartesian3.fromElements(
        satellite.positions_teme_meters[sample * 3],
        satellite.positions_teme_meters[sample * 3 + 1],
        satellite.positions_teme_meters[sample * 3 + 2],
        teme,
      );

      // Undefined before the Earth orientation data for this date is loaded;
      // skipping the sample is better than placing the satellite wrongly.
      if (
        !Cesium.Transforms.computeTemeToPseudoFixedMatrix(time, teme_to_fixed)
      ) {
        continue;
      }

      Cesium.Matrix3.multiplyByVector(teme_to_fixed, teme, fixed);
      position.addSample(time, fixed);
    }

    return position;
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
