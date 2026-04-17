<template>
  <div class="container">
    <h1>Welcome to Nuxt on Fugue!</h1>
    <p>This is a minimal Nuxt 3 application running on the Fugue FAAS platform.</p>
    <div class="info">
      <p><strong>Framework:</strong> Nuxt.js {{ nuxtVersion }}</p>
      <p><strong>Runtime:</strong> Node.js</p>
      <p><strong>Server:</strong> Nitro</p>
    </div>
  </div>
</template>

<script setup lang="ts">
const nuxtVersion = '3.x'
</script>

<style scoped>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  font-family: system-ui, -apple-system, sans-serif;
}

h1 {
  color: #00dc82;
  margin-bottom: 1rem;
}

.info {
  background: #f5f5f5;
  padding: 1.5rem;
  border-radius: 8px;
  margin-top: 2rem;
}

.info p {
  margin: 0.5rem 0;
}
</style>
