<template>
  <div>
    <UContainer>
      <header class="py-6 border-b border-gray-200 dark:border-gray-800">
        <nav class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <UIcon name="i-heroicons-bolt" class="w-8 h-8 text-primary" />
            <span class="text-xl font-bold">Fugue + Nuxt</span>
          </div>
          <div class="flex items-center gap-6">
            <NuxtLink to="/" class="text-gray-700 dark:text-gray-300 hover:text-primary transition-colors">
              Home
            </NuxtLink>
            <NuxtLink to="/about" class="text-gray-700 dark:text-gray-300 hover:text-primary transition-colors">
              About
            </NuxtLink>
            <NuxtLink to="/features" class="text-gray-700 dark:text-gray-300 hover:text-primary transition-colors">
              Features
            </NuxtLink>
            <NuxtLink to="/contact" class="text-gray-700 dark:text-gray-300 hover:text-primary transition-colors">
              Contact
            </NuxtLink>
          </div>
        </nav>
      </header>
    </UContainer>

    <main>
      <slot />
    </main>

    <footer class="border-t border-gray-200 dark:border-gray-800 mt-16">
      <UContainer>
        <div class="py-8 text-center text-sm text-gray-600 dark:text-gray-400">
          <p>&copy; {{ new Date().getFullYear() }} Fugue Nuxt Example. All rights reserved.</p>
        </div>
      </UContainer>
    </footer>
  </div>
</template>

<script setup lang="ts">
</script>
