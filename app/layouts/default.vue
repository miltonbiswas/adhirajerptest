<script setup lang="ts">
import AppSidebar from '~/components/layout/AppSidebar.vue'
import AppHeader from '~/components/layout/AppHeader.vue'
</script>

<template>
  <div class="flex h-dvh overflow-hidden bg-white">
    <!-- Sidebar: fixed-width, full-height layer -->
    <aside class="relative z-[80] h-full shrink-0">
      <AppSidebar />
    </aside>

    <!-- Application workspace -->
    <div class="relative flex min-h-0 min-w-0 flex-1 flex-col">
      <!-- Header stays above page content -->
      <div class="relative z-[70] shrink-0">
        <AppHeader />
      </div>

      <!-- Only the page content scrolls -->
      <main class="relative z-0 min-h-0 min-w-0 flex-1 overflow-x-hidden overflow-y-auto">
        <slot />
      </main>
    </div>
  </div>
</template>