<script lang="ts">
  import { AppRail, AppRailTile } from '@skeletonlabs/skeleton'
  import { writable, type Writable } from 'svelte/store'
  import { goto } from '$app/navigation'
  import { onDestroy } from 'svelte'

  const selectedTile: Writable<string> = writable('/keymaps')
  const unsubscribe = selectedTile.subscribe((route) => {
    goto(route)
  })

  onDestroy(() => unsubscribe())
</script>

<AppRail selected={selectedTile}>
  <svelte:fragment slot="lead">
    <!--        TODO: logo-->
  </svelte:fragment>

  <AppRailTile label="Keymaps" title="Keymaps" value="/keymaps">
    <!--        TODO: icon-->
  </AppRailTile>

  <AppRailTile label="Setlists" title="Setlists" value="/setlists">
    <!--        TODO: icon-->
  </AppRailTile>

  <svelte:fragment slot="trail">
    <AppRailTile label="Settings" title="Settings" value="/settings">
      <!--        TODO: icon-->
    </AppRailTile>
  </svelte:fragment>
</AppRail>
