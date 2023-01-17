<script lang="ts">
  import {AppRail, AppRailTile} from '@skeletonlabs/skeleton';
  import {writable, type Writable} from 'svelte/store';

  const selectedTile: Writable<number> = writable(1);

  // TODO: change route on selection change
</script>

<AppRail selected={selectedTile}>
  <svelte:fragment slot="lead">
    <!--        TODO: logo-->
  </svelte:fragment>

  <AppRailTile label="Edit" title="Edit" value={1}>
    <!--        TODO: icon-->
  </AppRailTile>

  <AppRailTile label="Play" title="Play" value={2}>
    <!--        TODO: icon-->
  </AppRailTile>

  <svelte:fragment slot="trail">
    <AppRailTile label="Settings" title="Settings" value={20}>
      <!--        TODO: icon-->
    </AppRailTile>
  </svelte:fragment>
</AppRail>
