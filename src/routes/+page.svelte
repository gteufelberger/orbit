<script lang="ts">
  import CesiumViewer from "$lib/CesiumViewer.svelte";
  import Controls from "$lib/Controls.svelte";

  let leftSidebarWidth = $state(150);
  let isDragging = $state(false);
  let inertialMode = $state(false);

  function handleMouseDown(): void {
    isDragging = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  function handleMouseMove(e: MouseEvent): void {
    if (!isDragging) return;
    leftSidebarWidth = e.clientX;
  }

  function handleMouseUp(): void {
    isDragging = false;
    document.body.style.cursor = "auto";
    document.body.style.userSelect = "auto";
  }
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={handleMouseUp} />

<div class="split-container">
  <div style={`width: ${leftSidebarWidth}px`}>
    <a
      class="source-link"
      href="https://github.com/gteufelberger/orbit"
      target="_blank"
      rel="noopener noreferrer">Source</a
    >
    <br />
    <Controls
      {inertialMode}
      onToggleInertialMode={() => (inertialMode = !inertialMode)}
    />
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="divider" onmousedown={handleMouseDown}></div>
  <div class="viewer-pane">
    <CesiumViewer {inertialMode} />
  </div>
</div>

<style>
  .split-container {
    display: flex;
    width: 100%;
    height: 100vh;
  }

  .divider {
    width: 8px;
    background: #ccc;
    cursor: col-resize;
  }
  .divider:hover {
    background: #999;
  }

  .viewer-pane {
    flex: 1;
  }
</style>
