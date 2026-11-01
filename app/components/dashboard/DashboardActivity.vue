<script setup lang="ts">
import {
  ArrowRight,
  Clock3,
  FileText,
  PackageSearch,
} from 'lucide-vue-next'
</script>

<template>
  <section class="grid grid-cols-1 gap-4 xl:grid-cols-2">
    <!-- Recent transactions -->
    <article
      class="overflow-hidden rounded-[18px] border border-[#E1E8F0] bg-white"
    >
      <header
        class="flex items-center justify-between gap-4 border-b border-[#E8EDF3] px-5 py-4"
      >
        <div>
          <h2 class="text-[14px] font-semibold text-[#253349]">
            Recent transactions
          </h2>

          <p class="mt-1 text-[12px] text-[#71839D]">
            Latest purchase and sales activity
          </p>
        </div>

        <div
          class="flex size-10 shrink-0 items-center justify-center rounded-xl border border-[#DCE5EF] bg-[#F5F8FC] text-[#647793]"
        >
          <FileText class="size-[18px]" :stroke-width="1.8" />
        </div>
      </header>

      <div
        class="flex min-h-[220px] flex-col items-center justify-center px-6 py-8 text-center"
      >
        <div
          class="flex size-12 items-center justify-center rounded-full border border-[#DCE5EF] bg-[#F5F8FC] text-[#71839D]"
        >
          <Clock3 class="size-5" :stroke-width="1.7" />
        </div>

        <h3 class="mt-4 text-[13px] font-semibold text-[#253349]">
          No transactions to display
        </h3>

        <p class="mt-1.5 max-w-[280px] text-[12px] leading-5 text-[#71839D]">
          Purchase and sales entries will appear here when transaction
          records are available.
        </p>

        <div class="mt-5 flex items-center gap-3">
          <NuxtLink
            to="/purchases"
            class="inline-flex items-center gap-1.5 rounded-lg border border-[#DCE5EF] bg-white px-3 py-2 text-[11px] font-medium text-[#34445D] transition hover:border-[#B5EED7] hover:bg-[#F4FCF8] hover:text-[#008F5B]"
          >
            Purchases
            <ArrowRight class="size-3.5" />
          </NuxtLink>

          <NuxtLink
            to="/sales"
            class="inline-flex items-center gap-1.5 rounded-lg bg-[#008F5B] px-3 py-2 text-[11px] font-medium text-white transition hover:bg-[#00784C]"
          >
            Sales
            <ArrowRight class="size-3.5" />
          </NuxtLink>
        </div>
      </div>
    </article>

    <!-- Inventory alerts -->
    <article
      class="overflow-hidden rounded-[18px] border border-[#E1E8F0] bg-white"
    >
      <header
        class="flex items-center justify-between gap-4 border-b border-[#E8EDF3] px-5 py-4"
      >
        <div>
          <h2 class="text-[14px] font-semibold text-[#253349]">
            Inventory alerts
          </h2>

          <p class="mt-1 text-[12px] text-[#71839D]">
            Low stock and approaching expiry
          </p>
        </div>

        <div
          class="flex size-10 shrink-0 items-center justify-center rounded-xl border border-[#B5EED7] bg-[#E5F9F0] text-[#008F5B]"
        >
          <PackageSearch class="size-[19px]" :stroke-width="1.8" />
        </div>
      </header>

      <div
        class="flex min-h-[220px] flex-col items-center justify-center px-6 py-8 text-center"
      >
        <div
          class="flex size-12 items-center justify-center rounded-full border border-[#B5EED7] bg-[#E5F9F0] text-[#008F5B]"
        >
          <PackageSearch class="size-5" :stroke-width="1.7" />
        </div>

        <h3 class="mt-4 text-[13px] font-semibold text-[#253349]">
          Inventory alerts are not loaded
        </h3>

        <p class="mt-1.5 max-w-[280px] text-[12px] leading-5 text-[#71839D]">
          Stock quantities and batch expiry dates are needed to identify
          products that require attention.
        </p>

        <NuxtLink
          to="/inventory"
          class="mt-5 inline-flex items-center gap-1.5 rounded-lg border border-[#B5EED7] bg-[#F4FCF8] px-3 py-2 text-[11px] font-medium text-[#008F5B] transition hover:bg-[#E5F9F0]"
        >
          View inventory
          <ArrowRight class="size-3.5" />
        </NuxtLink>
      </div>
    </article>
  </section>
</template>
