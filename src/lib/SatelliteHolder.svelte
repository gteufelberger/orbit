<script lang="ts">
  import type { Satellite } from "$lib/types";

  let { satellite }: { satellite: Satellite } = $props();
</script>

<pre>
    {JSON.stringify(satellite, null, 2)}
</pre>
