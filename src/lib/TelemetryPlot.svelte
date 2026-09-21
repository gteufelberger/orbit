<script lang="ts">
  import { AnnotationLine, LineChart, Spline } from "layerchart";
  import type { TelemetryChannel } from "$bindings/TelemetryChannel";
  import { playheadMinutes } from "$lib/simulation.svelte";

  let {
    channel,
    step_seconds,
    color = "#4a9eff",
  }: {
    channel: TelemetryChannel;
    step_seconds: number;
    color?: string;
  } = $props();

  // Elapsed minutes keeps the x axis numeric, so the default linear scale
  // applies and no d3 time scale has to be imported.
  let points = $derived(
    channel.values.map((value, index) => ({
      minutes: (index * step_seconds) / 60,
      value,
    })),
  );

  // Where the globe's clock currently sits, in the same units as the x axis.
  let playhead = $derived(playheadMinutes());

  /** The part of LayerChart's chart context the `marks` snippet below uses. */
  type MarksContext = {
    series: { visibleSeries: { key: string }[] };
  };
</script>

<figure>
  <figcaption>{channel.spec.label} ({channel.spec.unit})</figcaption>
  <div class="plot">
    <LineChart
      data={points}
      x="minutes"
      y="value"
      padding={{ left: 40, bottom: 20, top: 4, right: 6 }}
      series={[{ key: channel.spec.id, value: "value", color }]}
    >
      <!--
        Overriding `marks` replaces the default rendering, so the splines are
        redrawn here alongside the playhead. AnnotationLine takes a domain-space
        x and resolves it through the chart's own scale, which keeps the marker
        aligned with the data without measuring pixels.
      -->
      {#snippet marks({ context }: { context: MarksContext })}
        {#each context.series.visibleSeries as visible (visible.key)}
          <Spline seriesKey={visible.key} />
        {/each}
        {#if playhead !== null}
          <AnnotationLine x={playhead} stroke="#e4572e" stroke-width={1.5} />
        {/if}
      {/snippet}
    </LineChart>
  </div>
</figure>

<style>
  figure {
    margin: 0.75em 0 0;
  }

  figcaption {
    font-size: 0.75em;
    color: #666;
    margin-bottom: 0.2em;
  }

  /* LayerChart fills its container, so the height has to come from here. */
  .plot {
    height: 90px;
    font-size: 0.65em;
  }
</style>
