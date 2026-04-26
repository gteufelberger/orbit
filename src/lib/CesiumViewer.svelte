<script lang="ts">
  import { onMount } from "svelte";
  import * as Cesium from "cesium";
  import "cesium/Build/Cesium/Widgets/widgets.css";
  import { getWasm, type OrbitWasmModule } from "./wasm";

  let container: HTMLDivElement;
  let viewer: Cesium.Viewer;
  let lastClockTime: Cesium.JulianDate | null = null;
  let wasm: OrbitWasmModule;

  // ISS-like orbital elements
  const SEMI_MAJOR_AXIS = 6_371_000 + 400_000; // m (400 km altitude)
  const ECCENTRICITY = 0.0001;
  const INCLINATION = (51.6 * Math.PI) / 180; // radians
  const RAAN = 0;
  const ARG_PERIAPSIS = 0;
  const MEAN_ANOMALY_EPOCH = 0;
  const EPOCH_UNIX = 0; // reference epoch: Jan 1, 1970

  const UNIX_EPOCH_JD = Cesium.JulianDate.fromDate(new Date(0));

  // Must match the constants in lib.rs
  const EARTH_ROTATION_RATE = 7.292115e-5; // rad/s (sidereal)
  const GMST_J2000 = 4.894961212823756; // radians at J2000.0
  const UNIX_TO_J2000 = 946_728_000; // seconds from Unix epoch to J2000.0

  let { inertialMode }: { inertialMode: boolean } = $props();
  // Tracks the last tick time so we can apply an incremental rotation each frame.
  let inertialLastTime: Cesium.JulianDate | null = null;

  $effect(() => {
    // Reset the incremental reference when mode is toggled.
    inertialLastTime =
      inertialMode && viewer
        ? Cesium.JulianDate.clone(viewer.clock.currentTime)
        : null;
  });

  function cesiumTimeToUnix(jd: Cesium.JulianDate): number {
    return Cesium.JulianDate.secondsDifference(jd, UNIX_EPOCH_JD);
  }

  /**
   * Counter-rotate the camera by Earth's rotation over Δt so it stays
   * fixed relative to the stars. Applied incrementally each tick so that
   * user pan/zoom interactions are preserved between frames.
   */
  function applyInertialFrame(
    camera: Cesium.Camera,
    lastTime: Cesium.JulianDate,
    currentTime: Cesium.JulianDate,
  ): void {
    const dt = Cesium.JulianDate.secondsDifference(currentTime, lastTime);
    const rotation = Cesium.Matrix3.fromRotationZ(-EARTH_ROTATION_RATE * dt);

    camera.position = Cesium.Matrix3.multiplyByVector(
      rotation,
      camera.position,
      new Cesium.Cartesian3(),
    );
    camera.direction = Cesium.Matrix3.multiplyByVector(
      rotation,
      camera.direction,
      new Cesium.Cartesian3(),
    );
    camera.up = Cesium.Matrix3.multiplyByVector(
      rotation,
      camera.up,
      new Cesium.Cartesian3(),
    );
  }

  onMount(() => {
    if (!container) return;

    (async () => {
      wasm = await getWasm();

      viewer = new Cesium.Viewer(container, {});

      // Satellite position using Keplerian mechanics
      viewer.entities.add({
        position: new Cesium.CallbackPositionProperty(
          () => {
            const t = cesiumTimeToUnix(viewer.clock.currentTime);
            const pos = wasm.calculate_position_keplerian(
              SEMI_MAJOR_AXIS,
              ECCENTRICITY,
              INCLINATION,
              RAAN,
              ARG_PERIAPSIS,
              MEAN_ANOMALY_EPOCH,
              EPOCH_UNIX,
              t,
            );
            return new Cesium.Cartesian3(pos[0], pos[1], pos[2]);
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

      // Orbit path: recalculated each tick so ECEF positions stay correct
      viewer.entities.add({
        polyline: {
          positions: new Cesium.CallbackProperty(() => {
            const t = cesiumTimeToUnix(viewer.clock.currentTime);
            const flat = wasm.calculate_orbit_path(
              SEMI_MAJOR_AXIS,
              ECCENTRICITY,
              INCLINATION,
              RAAN,
              ARG_PERIAPSIS,
              MEAN_ANOMALY_EPOCH,
              EPOCH_UNIX,
              t,
              360,
              true, // inertial: WASM returns ECI; we apply one rotation below
            );
            // Rotate all ECI points to ECEF using the current GMST, so the
            // orbital ellipse appears fixed in space while Earth rotates under it.
            const gmst = GMST_J2000 + EARTH_ROTATION_RATE * (t - UNIX_TO_J2000);
            const eciToFixed = Cesium.Matrix3.fromRotationZ(-gmst);
            const positions: Cesium.Cartesian3[] = [];
            for (let i = 0; i < flat.length; i += 3) {
              positions.push(
                Cesium.Matrix3.multiplyByVector(
                  eciToFixed,
                  new Cesium.Cartesian3(flat[i], flat[i + 1], flat[i + 2]),
                  new Cesium.Cartesian3(),
                ),
              );
            }
            return positions;
          }, false),
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
    if (inertialMode && inertialLastTime) {
      applyInertialFrame(viewer.camera, inertialLastTime, currentTime);
    }
    inertialLastTime = inertialMode
      ? Cesium.JulianDate.clone(currentTime)
      : null;
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
