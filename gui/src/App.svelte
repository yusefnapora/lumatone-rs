
<script>
	import { AppShell, AppBar, GradientHeading } from '@brainandbones/skeleton';
</script>

<AppShell slotSidebarLeft="bg-surface-500/5 w-56 p-4">
	<!-- Header -->
	<svelte:fragment slot="header">
    <AppBar>
      <svelte:fragment slot="lead">
        <h1>Skeleton</h1>
      </svelte:fragment>
      <svelte:fragment slot="trail">
        <a class="btn btn-sm" href="https://github.com/" target="_blank" rel="noreferrer">GitHub</a>
      </svelte:fragment>
    </AppBar>
  </svelte:fragment>
	<!-- Sidebar -->
	<svelte:fragment slot="sidebarLeft">
    <nav class="list-nav">
      <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/about">About</a></li>
      </ul>
    </nav>
  </svelte:fragment>
	<!-- Page Content -->
	<div id="page" class="container mx-auto p-8 space-y-8">
    <GradientHeading tag="h1" direction="bg-gradient-to-br" from="from-primary-500" to="to-accent-500">
      Homepage
    </GradientHeading>
    <p>Lorem ipsum dolor sit amet consectetur adipisicing elit.</p>
    <hr />
    <section class="card card-body">
      <p>Lorem ipsum dolor sit amet consectetur adipisicing elit.</p>
    </section>
    <hr />
    <section class="flex space-x-2">
        <a class="btn btn-filled-primary" href="https://kit.svelte.dev/" target="_blank" rel="noreferrer">SvelteKit</a>
        <a class="btn btn-filled-accent" href="https://tailwindcss.com/" target="_blank" rel="noreferrer">Tailwind</a>
        <a class="btn btn-filled-tertiary" href="https://github.com/" target="_blank" rel="noreferrer">GitHub</a>
      </section>
  </div>
</AppShell>
