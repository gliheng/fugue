// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  future: {
    compatibilityVersion: 4
  },
  compatibilityDate: '2024-01-01',
  devtools: { enabled: false },
  nitro: {
    preset: 'node-server'
  }
})
