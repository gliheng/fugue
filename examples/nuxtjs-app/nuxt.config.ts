// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  modules: ['@nuxt/ui', 'nitro-cloudflare-dev'],
  css: ['~/assets/style.css'],
  nitro: {
    preset: "cloudflare_module",

    cloudflare: {
      deployConfig: true,
      nodeCompat: true
    }
  },
  fonts: {
    provider: 'bunny'
  }
})
