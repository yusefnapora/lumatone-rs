<script>
  import '@skeletonlabs/skeleton/themes/theme-modern.css'
  import '@skeletonlabs/skeleton/styles/all.css'
  import '../app.postcss'

  import { AppShell } from '@skeletonlabs/skeleton'
  import RailNav from '../ui/RailNav.svelte'
</script>

<AppShell slotSidebarLeft="bg-surface-500/5">
  <!-- Header -->
  <svelte:fragment slot="header" />

  <!-- Sidebar -->
  <svelte:fragment slot="sidebarLeft">
    <RailNav />
  </svelte:fragment>

  <div id="page" class="container mx-auto p-8 space-y-8">
    <slot />
  </div>
</AppShell>
