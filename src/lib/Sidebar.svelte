<script lang="ts">
  import Controls from "./Controls.svelte";
  import SatelliteList from "./SatelliteList.svelte";
</script>

<div class="sidebar">
  <a
    class="source-link"
    href="https://github.com/gteufelberger/orbit"
    target="_blank"
    rel="noopener noreferrer">Source</a
  >

  <Controls />
  <SatelliteList />
</div>

<style>
  /* Custom properties inherit into the child components, so the palette lives
     here while each component keeps its own layout rules. */
  .sidebar {
    --fg: #e8ecf1;
    --fg-dim: #8b97a8;
    --line: #2b3546;
    --raised: #1d2634;
    --accent: #4a9eff;

    height: 100%;
    box-sizing: border-box;
    padding: 0.75rem;
    overflow-y: auto;
    background: #141b26;
    color: var(--fg);
    font:
      0.875rem/1.45 system-ui,
      sans-serif;
  }

  .source-link {
    color: var(--fg-dim);
    font-size: 0.8em;
  }
</style>
