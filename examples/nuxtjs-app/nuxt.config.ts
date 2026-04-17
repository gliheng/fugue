// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-01-01',
  devtools: { enabled: false },
  modules: ['@nuxt/ui'],
  css: ['~/assets/style.css'],
  nitro: {
    preset: 'node-server'
  },
  fonts: {
    provider: 'bunny'
  }
})
