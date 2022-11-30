<script>
  import {AppShell, GradientHeading} from '@brainandbones/skeleton';
  import RailNav from "./ui/RailNav.svelte";
  import Wheel from "./ui/wheel/Wheel.svelte";
</script>

<AppShell slotSidebarLeft="bg-surface-500/5">
  <!-- Header -->
  <svelte:fragment slot="header">
  </svelte:fragment>

  <!-- Sidebar -->
  <svelte:fragment slot="sidebarLeft">
    <RailNav/>
  </svelte:fragment>

  <!-- Page Content
  TODO: add router
  -->
  <div id="page" class="container mx-auto p-8 space-y-8">
    <GradientHeading tag="h2" direction="bg-gradient-to-r" from="from-red-500" to="to-violet-500">
      Preset Editor
    </GradientHeading>
    <Wheel />
  </div>
</AppShell>
