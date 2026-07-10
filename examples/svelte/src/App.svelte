<script lang="ts">
  let pkg: { name: string; description: string; version: string } | null = $state(null);

  $effect(() => {
    fetch('/api/v1/packages/vite')
      .then((res) => res.json())
      .then((data) => (pkg = data))
      .catch(() => {});
  });
</script>

<div>
  <h1>Dash + Svelte</h1>
  <p>Welcome to the Dash registry Svelte example.</p>
  {#if pkg}
    <div>
      <h2>{pkg.name}</h2>
      <p>{pkg.description}</p>
      <p>Latest version: {pkg.version}</p>
    </div>
  {/if}
</div>
