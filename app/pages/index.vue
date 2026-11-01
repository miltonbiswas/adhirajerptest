
<script setup lang="ts">
import { ArrowRight } from 'lucide-vue-next'

import DashboardStats from '~/components/dashboard/DashboardStats.vue'
import DashboardQuickActions from '~/components/dashboard/DashboardQuickActions.vue'
import DashboardActivity from '~/components/dashboard/DashboardActivity.vue'

useHead({
  title: 'Dashboard | AdhirajPOS',
})

useSeoMeta({
  description: 'AdhirajPOS wholesale business dashboard.',
})
</script>

<template>
  <div class="min-h-full bg-[#F7F9FC]">
    <div class="mx-auto w-full max-w-[1800px]">

      <!-- Dashboard statistics -->
      <section class="px-4 pt-5 sm:px-6 lg:px-7">
        <DashboardStats />
      </section>

      <!-- Quick actions -->
      <section class="px-4 pt-7 sm:px-6 lg:px-7">
        <DashboardQuickActions />
      </section>

      <!-- Business activity -->
      <section class="px-4 pb-6 pt-8 sm:px-6 lg:px-7">
        <div class="mb-4 flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2
              class="text-[16px] font-semibold tracking-[-0.2px] text-[#101828]"
            >
              Business activity
            </h2>

            <p class="mt-1 text-[13px] text-[#667085]">
              Recent transactions and inventory updates.
            </p>
          </div>

          <NuxtLink
            to="/reports"
            class="inline-flex items-center gap-2 rounded-lg border border-[#E4E7EC] bg-white px-3 py-2 text-[12px] font-medium text-[#344054] transition hover:border-[#CBD5E1] hover:bg-[#F9FAFB]"
          >
            View reports
            <ArrowRight class="size-3.5" />
          </NuxtLink>
        </div>

        <DashboardActivity />
      </section>

      <!-- Footer -->
      <footer
        class="mx-4 flex flex-col gap-2 border-t border-[#EAECF0] py-4 text-[11px] text-[#667085] sm:mx-6 sm:flex-row sm:items-center sm:justify-between lg:mx-7"
      >
        <span>AdhirajPOS</span>
        <span>Wholesale Management System</span>
      </footer>

    </div>
  </div>
</template>
