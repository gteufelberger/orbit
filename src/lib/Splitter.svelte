<script lang="ts">
  import type { Snippet } from "svelte";

  // Width of the left content
  let left_width = $state(150);

  // Left / right content from parent
  let {
    left_content: left,
    right_content: right,
  }: {
    left_content: Snippet;
    right_content: Snippet;
  } = $props();
  let isDragging = $state(false);

  function handleMouseDown(): void {
    isDragging = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  function handleMouseMove(e: MouseEvent): void {
    if (!isDragging) return;
    left_width = e.clientX;
  }

  function handleMouseUp(): void {
    isDragging = false;
    document.body.style.cursor = "auto";
    document.body.style.userSelect = "auto";
  }
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={handleMouseUp} />

<div class="split-container">
  <div style={`width: ${left_width}px`}>
    {@render left()}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="divider" onmousedown={handleMouseDown}></div>
  <div class="viewer-pane">
    {@render right()}
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
