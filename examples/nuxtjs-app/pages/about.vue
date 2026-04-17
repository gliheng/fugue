<template>
  <UContainer>
    <div class="py-16 max-w-4xl mx-auto">
      <h1 class="text-4xl font-bold mb-8">About Fugue + Nuxt</h1>

      <div class="prose dark:prose-invert max-w-none">
        <UCard class="mb-8">
          <template #header>
            <h2 class="text-2xl font-semibold">What is Fugue?</h2>
          </template>
          <p class="text-gray-600 dark:text-gray-400 mb-4">
            Fugue is a modern serverless platform designed for deploying full-stack applications with minimal configuration. It provides fast cold starts, automatic scaling, and seamless integration with popular frameworks.
          </p>
          <p class="text-gray-600 dark:text-gray-400">
            Built on top of cutting-edge runtime technologies, Fugue enables developers to focus on building features rather than managing infrastructure.
          </p>
        </UCard>

        <UCard class="mb-8">
          <template #header>
            <h2 class="text-2xl font-semibold">Why Nuxt 3?</h2>
          </template>
          <p class="text-gray-600 dark:text-gray-400 mb-4">
            Nuxt 3 is the latest version of the popular Vue.js framework, offering:
          </p>
          <ul class="space-y-2 text-gray-600 dark:text-gray-400">
            <li class="flex items-start gap-2">
              <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5" />
              <span>Server-side rendering (SSR) for better SEO and performance</span>
            </li>
            <li class="flex items-start gap-2">
              <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5" />
              <span>Nitro server engine for universal deployment</span>
            </li>
            <li class="flex items-start gap-2">
              <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5" />
              <span>Auto-imports and file-based routing</span>
            </li>
            <li class="flex items-start gap-2">
              <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5" />
              <span>TypeScript support out of the box</span>
            </li>
            <li class="flex items-start gap-2">
              <UIcon name="i-heroicons-check-circle" class="w-5 h-5 text-primary mt-0.5" />
              <span>Built-in API routes for full-stack development</span>
            </li>
          </ul>
        </UCard>

        <UCard>
          <template #header>
            <h2 class="text-2xl font-semibold">Perfect Together</h2>
          </template>
          <p class="text-gray-600 dark:text-gray-400 mb-4">
            The combination of Nuxt 3 and Fugue provides an excellent developer experience:
          </p>
          <div class="grid md:grid-cols-2 gap-4">
            <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
              <h3 class="font-semibold mb-2 flex items-center gap-2">
                <UIcon name="i-heroicons-lightning-bolt" class="w-5 h-5 text-primary" />
                Fast Deployment
              </h3>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Deploy your Nuxt app in seconds with automatic optimization
              </p>
            </div>
            <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
              <h3 class="font-semibold mb-2 flex items-center gap-2">
                <UIcon name="i-heroicons-arrow-trending-up" class="w-5 h-5 text-primary" />
                Auto Scaling
              </h3>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Handle traffic spikes without manual intervention
              </p>
            </div>
            <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
              <h3 class="font-semibold mb-2 flex items-center gap-2">
                <UIcon name="i-heroicons-globe-alt" class="w-5 h-5 text-primary" />
                Global Edge
              </h3>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Serve content from the edge for minimal latency
              </p>
            </div>
            <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
              <h3 class="font-semibold mb-2 flex items-center gap-2">
                <UIcon name="i-heroicons-currency-dollar" class="w-5 h-5 text-primary" />
                Cost Efficient
              </h3>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Pay only for what you use with serverless pricing
              </p>
            </div>
          </div>
        </UCard>
      </div>

      <div class="mt-12 text-center">
        <UButton size="lg" to="/features">
          Explore Features
          <template #trailing>
            <UIcon name="i-heroicons-arrow-right" />
          </template>
        </UButton>
      </div>
    </div>
  </UContainer>
</template>

<script setup lang="ts">
</script>
