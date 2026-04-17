<template>
  <UContainer>
    <div class="py-16">
      <div class="text-center mb-12">
        <h1 class="text-4xl font-bold mb-4">Platform Features</h1>
        <p class="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
          Discover what makes Fugue + Nuxt the perfect combination for modern web applications
        </p>
      </div>

      <!-- Feature Tabs -->
      <UTabs :items="tabs" class="mb-12">
        <template #runtime="{ item }">
          <div class="py-8">
            <div class="grid md:grid-cols-2 gap-8">
              <div>
                <h3 class="text-2xl font-bold mb-4">{{ item.title }}</h3>
                <p class="text-gray-600 dark:text-gray-400 mb-6">
                  {{ item.description }}
                </p>
                <ul class="space-y-3">
                  <li v-for="feature in item.features" :key="feature" class="flex items-start gap-2">
                    <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                    <span class="text-gray-700 dark:text-gray-300">{{ feature }}</span>
                  </li>
                </ul>
              </div>
              <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
                <pre class="text-sm overflow-x-auto"><code>{{ item.code }}</code></pre>
              </div>
            </div>
          </div>
        </template>

        <template #rendering="{ item }">
          <div class="py-8">
            <div class="grid md:grid-cols-2 gap-8">
              <div>
                <h3 class="text-2xl font-bold mb-4">{{ item.title }}</h3>
                <p class="text-gray-600 dark:text-gray-400 mb-6">
                  {{ item.description }}
                </p>
                <ul class="space-y-3">
                  <li v-for="feature in item.features" :key="feature" class="flex items-start gap-2">
                    <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                    <span class="text-gray-700 dark:text-gray-300">{{ feature }}</span>
                  </li>
                </ul>
              </div>
              <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
                <pre class="text-sm overflow-x-auto"><code>{{ item.code }}</code></pre>
              </div>
            </div>
          </div>
        </template>

        <template #routing="{ item }">
          <div class="py-8">
            <div class="grid md:grid-cols-2 gap-8">
              <div>
                <h3 class="text-2xl font-bold mb-4">{{ item.title }}</h3>
                <p class="text-gray-600 dark:text-gray-400 mb-6">
                  {{ item.description }}
                </p>
                <ul class="space-y-3">
                  <li v-for="feature in item.features" :key="feature" class="flex items-start gap-2">
                    <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                    <span class="text-gray-700 dark:text-gray-300">{{ feature }}</span>
                  </li>
                </ul>
              </div>
              <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
                <pre class="text-sm overflow-x-auto"><code>{{ item.code }}</code></pre>
              </div>
            </div>
          </div>
        </template>

        <template #api="{ item }">
          <div class="py-8">
            <div class="grid md:grid-cols-2 gap-8">
              <div>
                <h3 class="text-2xl font-bold mb-4">{{ item.title }}</h3>
                <p class="text-gray-600 dark:text-gray-400 mb-6">
                  {{ item.description }}
                </p>
                <ul class="space-y-3">
                  <li v-for="feature in item.features" :key="feature" class="flex items-start gap-2">
                    <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                    <span class="text-gray-700 dark:text-gray-300">{{ feature }}</span>
                  </li>
                </ul>
              </div>
              <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
                <pre class="text-sm overflow-x-auto"><code>{{ item.code }}</code></pre>
              </div>
            </div>
          </div>
        </template>
      </UTabs>

      <!-- Stats -->
      <div class="grid md:grid-cols-4 gap-6 mb-12">
        <UCard>
          <div class="text-center">
            <div class="text-3xl font-bold text-primary mb-2">&lt;100ms</div>
            <div class="text-sm text-gray-600 dark:text-gray-400">Cold Start Time</div>
          </div>
        </UCard>
        <UCard>
          <div class="text-center">
            <div class="text-3xl font-bold text-primary mb-2">99.9%</div>
            <div class="text-sm text-gray-600 dark:text-gray-400">Uptime SLA</div>
          </div>
        </UCard>
        <UCard>
          <div class="text-center">
            <div class="text-3xl font-bold text-primary mb-2">Global</div>
            <div class="text-sm text-gray-600 dark:text-gray-400">Edge Network</div>
          </div>
        </UCard>
        <UCard>
          <div class="text-center">
            <div class="text-3xl font-bold text-primary mb-2">Auto</div>
            <div class="text-sm text-gray-600 dark:text-gray-400">Scaling</div>
          </div>
        </UCard>
      </div>

      <div class="text-center">
        <UButton size="lg" to="/contact">
          Get Started
          <template #trailing>
            <UIcon name="i-heroicons-arrow-right" />
          </template>
        </UButton>
      </div>
    </div>
  </UContainer>
</template>

<script setup lang="ts">
const tabs = [
  {
    key: 'runtime',
    label: 'Runtime',
    icon: 'i-heroicons-cpu-chip',
    title: 'Nitro Server Engine',
    description: 'Powered by Nitro, Nuxt 3 provides a universal server engine that works everywhere.',
    features: [
      'Cross-platform deployment support',
      'Automatic code splitting',
      'Built-in caching strategies',
      'Hot module replacement in development'
    ],
    code: `// nitro.config.ts
export default defineNitroConfig({
  preset: 'node-server',
  serveStatic: true,
  compressPublicAssets: true
})`
  },
  {
    key: 'rendering',
    label: 'Rendering',
    icon: 'i-heroicons-sparkles',
    title: 'Flexible Rendering Modes',
    description: 'Choose between SSR, SSG, or hybrid rendering based on your needs.',
    features: [
      'Server-side rendering (SSR)',
      'Static site generation (SSG)',
      'Incremental static regeneration',
      'Client-side rendering fallback'
    ],
    code: `// nuxt.config.ts
export default defineNuxtConfig({
  ssr: true,
  routeRules: {
    '/': { prerender: true },
    '/api/**': { cors: true }
  }
})`
  },
  {
    key: 'routing',
    label: 'Routing',
    icon: 'i-heroicons-map',
    title: 'File-Based Routing',
    description: 'Automatic route generation based on your file structure.',
    features: [
      'Zero configuration routing',
      'Dynamic route parameters',
      'Nested routes support',
      'Middleware and guards'
    ],
    code: `// pages/blog/[id].vue
<script setup>
const route = useRoute()
const { data } = await useFetch(
  \\\`/api/posts/\\\${route.params.id}\\\`
)
<\/script>`
  },
  {
    key: 'api',
    label: 'API Routes',
    icon: 'i-heroicons-code-bracket',
    title: 'Built-in API Routes',
    description: 'Create full-stack applications with server API routes.',
    features: [
      'File-based API routing',
      'TypeScript support',
      'Request validation',
      'Automatic serialization'
    ],
    code: `// server/api/hello.ts
export default defineEventHandler(() => {
  return {
    message: 'Hello from Fugue!',
    timestamp: new Date()
  }
})`
  }
]

</script>
