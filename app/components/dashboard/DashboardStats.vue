<script setup lang="ts">
import {
  Boxes,
  IndianRupee,
  ShoppingBag,
  Wallet,
} from 'lucide-vue-next'

interface Props {
  totalProducts?: number | null
  stockValuation?: number | null
  todaySales?: number | null
  outstanding?: number | null
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  totalProducts: null,
  stockValuation: null,
  todaySales: null,
  outstanding: null,
  loading: false,
})

const currency = new Intl.NumberFormat('en-IN', {
  style: 'currency',
  currency: 'INR',
  maximumFractionDigits: 2,
})

const number = new Intl.NumberFormat('en-IN')

const stats = computed(() => [
  {
    label: 'Total products',
    value: props.totalProducts === null
      ? '—'
      : number.format(props.totalProducts),
    icon: Boxes,
  },
  {
    label: 'Stock valuation',
    value: props.stockValuation === null
      ? '—'
      : currency.format(props.stockValuation),
    icon: IndianRupee,
  },
  {
    label: "Today's sales",
    value: props.todaySales === null
      ? '—'
      : currency.format(props.todaySales),
    icon: ShoppingBag,
  },
  {
    label: 'Outstanding',
    value: props.outstanding === null
      ? '—'
      : currency.format(props.outstanding),
    icon: Wallet,
  },
])
</script>

<template>
  <section
    aria-label="Business statistics"
    class="overflow-hidden rounded-[20px] border border-[#E1E8F0] bg-[#F5F8FC]"
  >
    <div class="grid grid-cols-1 divide-y divide-[#DFE7F0] sm:grid-cols-2 sm:divide-x sm:divide-y-0 xl:grid-cols-4">
      <article
        v-for="stat in stats"
        :key="stat.label"
        class="min-w-0 px-6 py-5 transition-colors duration-150 hover:bg-white/70 sm:px-6 sm:py-6"
      >
        <div
          class="flex size-10 items-center justify-center rounded-full border border-[#9DEBCB] bg-[#DDF8ED] text-[#00A86B]"
        >
          <component
            :is="stat.icon"
            class="size-[19px]"
            :stroke-width="1.8"
            aria-hidden="true"
          />
        </div>

        <p class="mt-3 text-[12px] font-medium uppercase tracking-wide text-[#71839D]">
          {{ stat.label }}
        </p>

        <div class="mt-1.5 min-h-8">
          <div
            v-if="props.loading"
            class="mt-1 h-7 w-28 animate-pulse rounded-md bg-[#E3EAF2]"
            aria-label="Loading statistic"
          />

          <p
            v-else
            class="break-words text-[25px] font-semibold leading-8 tracking-[-0.7px] text-[#172235] sm:text-[26px]"
          >
            {{ stat.value }}
          </p>
        </div>
      </article>
    </div>
  </section>
</template>
