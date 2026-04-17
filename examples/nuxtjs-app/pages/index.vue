<template>
  <UContainer>
    <div class="py-16">
      <!-- Hero Section -->
      <div class="text-center mb-16">
        <UBadge color="primary" variant="subtle" class="mb-4">
          Nuxt 3 + Nitro Server
        </UBadge>
        <h1 class="text-5xl font-bold mb-6">
          Deploy Nuxt on the <span class="text-primary">Fugue</span> Platform
        </h1>
        <p class="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mb-8">
          A modern full-stack application showcasing Nuxt 3 running on the Fugue serverless platform with server-side rendering and API routes.
        </p>
        <div class="flex gap-4 justify-center">
          <UButton size="lg" to="/features">
            Explore Features
            <template #trailing>
              <UIcon name="i-heroicons-arrow-right" />
            </template>
          </UButton>
          <UButton size="lg" color="gray" variant="ghost" to="/about">
            Learn More
          </UButton>
        </div>
      </div>

      <!-- Features Grid -->
      <div class="grid md:grid-cols-3 gap-6 mb-16">
        <UCard>
          <template #header>
            <div class="flex items-center gap-3">
              <UIcon name="i-heroicons-rocket-launch" class="w-6 h-6 text-primary" />
              <h3 class="text-lg font-semibold">Server-Side Rendering</h3>
            </div>
          </template>
          <p class="text-gray-600 dark:text-gray-400">
            Full SSR support with Nitro server for optimal performance and SEO.
          </p>
        </UCard>

        <UCard>
          <template #header>
            <div class="flex items-center gap-3">
              <UIcon name="i-heroicons-bolt" class="w-6 h-6 text-primary" />
              <h3 class="text-lg font-semibold">Fast Cold Starts</h3>
            </div>
          </template>
          <p class="text-gray-600 dark:text-gray-400">
            Optimized for serverless with minimal startup time and efficient resource usage.
          </p>
        </UCard>

        <UCard>
          <template #header>
            <div class="flex items-center gap-3">
              <UIcon name="i-heroicons-code-bracket" class="w-6 h-6 text-primary" />
              <h3 class="text-lg font-semibold">API Routes</h3>
            </div>
          </template>
          <p class="text-gray-600 dark:text-gray-400">
            Built-in API routes with Nitro for full-stack development in one framework.
          </p>
        </UCard>
      </div>

      <!-- Info Panel -->
      <UCard>
        <template #header>
          <h2 class="text-2xl font-bold">Current Deployment</h2>
        </template>
        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Current Time</p>
            <p class="font-mono text-sm">{{ currentTime }}</p>
          </div>
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Runtime</p>
            <p class="font-mono text-sm">Node.js / Nitro</p>
          </div>
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Framework</p>
            <p class="font-mono text-sm">Nuxt 3</p>
          </div>
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Platform</p>
            <p class="font-mono text-sm">Fugue Serverless</p>
          </div>
        </div>
      </UCard>
    </div>
  </UContainer>
</template>

<script setup lang="ts">
const currentTime = ref('')

onMounted(() => {
  currentTime.value = new Date().toISOString()
  setInterval(() => {
    currentTime.value = new Date().toISOString()
  }, 1000)
})
</script>
