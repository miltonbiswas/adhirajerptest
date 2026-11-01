export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',

  devtools: {
    enabled: true,
  },

  ssr: false,

  modules: ['@nuxt/ui'],

  css: ['~/assets/css/main.css'],

  app: {
    head: {
      title: 'AdhirajPOS',
      titleTemplate: '%s | AdhirajPOS',

      meta: [
        {
          name: 'description',
          content:
            'AdhirajPOS - Pharmacy management, inventory, billing and business operations.',
        },
        {
          name: 'application-name',
          content: 'AdhirajPOS',
        },
        {
          name: 'theme-color',
          content: '#ffffff',
        },
      ],

      link: [
        {
          rel: 'icon',
          type: 'image/png',
          href: '/favicon.png',
        },
      ],
    },
  },

  vite: {
    clearScreen: false,

    server: {
      strictPort: true,
    },
  },
})
