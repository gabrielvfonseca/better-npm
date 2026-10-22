<script setup lang="ts">
import { ref, onMounted } from 'vue';

interface Package {
  name: string;
  description: string;
  version: string;
}

const title = 'Bnpm + Vue';
const pkg = ref<Package | null>(null);

onMounted(async () => {
  try {
    const res = await fetch('/api/v1/packages/express');
    pkg.value = await res.json();
  } catch {
    // ignore
  }
});
</script>

<template>
  <div>
    <h1>{{ title }}</h1>
    <p>Welcome to the Bnpm registry Vue example.</p>
    <div v-if="pkg">
      <h2>{{ pkg.name }}</h2>
      <p>{{ pkg.description }}</p>
      <p>Latest version: {{ pkg.version }}</p>
    </div>
  </div>
</template>
