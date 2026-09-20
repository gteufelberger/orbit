<script lang="ts">
  import CesiumViewer from "$lib/CesiumViewer.svelte";
  import Sidebar from "$lib/Sidebar.svelte";
  import Splitter from "$lib/Splitter.svelte";
</script>

<Splitter>
  {#snippet left_content()}
    <Sidebar />
  {/snippet}
  {#snippet right_content()}
    <CesiumViewer />
  {/snippet}
</Splitter>
