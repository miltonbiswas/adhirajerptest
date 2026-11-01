<script setup lang="ts">
import {
  ArrowDownToLine,
  ArrowRight,
  ArrowUpFromLine,
  PackagePlus,
  ScanBarcode,
} from 'lucide-vue-next'

const actions = [
  {
    label: 'New Purchase',
    description: 'Record a supplier invoice',
    icon: ArrowDownToLine,
    to: '/purchases',
  },
  {
    label: 'New Sale',
    description: 'Create a sales invoice',
    icon: ArrowUpFromLine,
    to: '/sales',
  },
  {
    label: 'Add Product',
    description: 'Add a product to the catalogue',
    icon: PackagePlus,
    to: '/products',
  },
  {
    label: 'Stock Management',
    description: 'Check batches and stock levels',
    icon: ScanBarcode,
    to: '/inventory',
  },
]
</script>

<template>
  <section aria-labelledby="quick-actions-heading">
    <div class="mb-4">
      <h2
        id="quick-actions-heading"
        class="text-[15px] font-semibold tracking-[-0.2px] text-[#172235]"
      >
        Quick actions
      </h2>

      <p class="mt-1 text-[12px] text-[#71839D]">
        Shortcuts to your daily tasks
      </p>
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
      <NuxtLink
        v-for="action in actions"
        :key="action.label"
        :to="action.to"
        class="group flex min-w-0 items-center gap-3.5 rounded-[15px] border border-[#E1E8F0] bg-white p-4 transition duration-200 hover:border-[#A8E7CD] hover:bg-[#FBFEFC] hover:shadow-[0_3px_12px_rgba(16,40,30,0.045)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00A86B] focus-visible:ring-offset-2"
      >
        <div
          class="flex size-11 shrink-0 items-center justify-center rounded-xl border border-[#B5EED7] bg-[#E5F9F0] text-[#008F5B] transition-colors duration-200 group-hover:border-[#00A86B] group-hover:bg-[#00A86B] group-hover:text-white"
        >
          <component
            :is="action.icon"
            class="size-[19px]"
            :stroke-width="1.8"
            aria-hidden="true"
          />
        </div>

        <div class="min-w-0 flex-1">
          <p class="truncate text-[13px] font-semibold text-[#253349]">
            {{ action.label }}
          </p>

          <p class="mt-1 text-[11px] leading-4 text-[#71839D]">
            {{ action.description }}
          </p>
        </div>

        <ArrowRight
          class="size-4 shrink-0 text-[#A0AEC0] opacity-0 transition duration-200 group-hover:translate-x-0.5 group-hover:opacity-100 group-focus-visible:opacity-100"
          aria-hidden="true"
        />
      </NuxtLink>
    </div>
  </section>
</template>
