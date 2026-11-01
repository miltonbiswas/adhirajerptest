<script setup lang="ts">
import type { Party, PartyRole } from '~/composables/useApi'

useSeoMeta({
  title: 'Customers | AdhirajPOS',
})

const { getParties, deactivateParty } = useApi()

const customers = ref<Party[]>([])
const loading = ref(true)
const pageError = ref('')
const search = ref('')
const activeFilter = ref<'all' | 'active' | 'inactive'>('active')

const page = ref(1)
const pageSize = 25
const total = ref(0)

const modalOpen = ref(false)
const selectedParty = ref<Party | null>(null)

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / pageSize)),
)

const startRecord = computed(() =>
  total.value === 0 ? 0 : (page.value - 1) * pageSize + 1,
)

const endRecord = computed(() =>
  Math.min(page.value * pageSize, total.value),
)

let searchTimer: ReturnType<typeof setTimeout> | undefined
let requestSequence = 0

async function loadCustomers() {
  const sequence = ++requestSequence

  loading.value = true
  pageError.value = ''

  try {
    const response = await getParties({
      search: search.value.trim() || undefined,
      role: 'CUSTOMER' as PartyRole,
      page: page.value,
      limit: pageSize,
      active:
        activeFilter.value === 'all'
          ? undefined
          : activeFilter.value === 'active',
    })

    if (sequence !== requestSequence) return

    customers.value = response.data
    total.value = response.total
  } catch (error) {
    if (sequence !== requestSequence) return

    pageError.value =
      error instanceof Error
        ? error.message
        : 'Unable to load customers. Please try again.'

    customers.value = []
    total.value = 0
  } finally {
    if (sequence === requestSequence) {
      loading.value = false
    }
  }
}

function handleSearch() {
  if (searchTimer) clearTimeout(searchTimer)

  searchTimer = setTimeout(() => {
    page.value = 1
    loadCustomers()
  }, 300)
}

watch(activeFilter, () => {
  page.value = 1
  loadCustomers()
})

watch(page, () => {
  loadCustomers()
})

function openCreateModal() {
  selectedParty.value = null
  modalOpen.value = true
}

function openEditModal(customer: Party) {
  selectedParty.value = customer
  modalOpen.value = true
}

async function handleSaved() {
  modalOpen.value = false
  selectedParty.value = null
  await loadCustomers()
}

async function handleDeactivate(customer: Party) {
  if (!customer.is_active) return

  const confirmed = window.confirm(
    `Deactivate "${customer.name}"? This will not permanently delete the party.`,
  )

  if (!confirmed) return

  pageError.value = ''

  try {
    await deactivateParty(customer.id)

    if (customers.value.length === 1 && page.value > 1) {
      page.value--
    } else {
      await loadCustomers()
    }
  } catch (error) {
    pageError.value =
      error instanceof Error
        ? error.message
        : 'Unable to deactivate this customer.'
  }
}

function formatCurrency(value: number | string | null | undefined) {
  const amount = Number(value ?? 0)

  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2,
  }).format(Number.isFinite(amount) ? amount : 0)
}

onMounted(() => {
  loadCustomers()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
})
</script>

<template>
  <main class="min-h-full bg-neutral-50 p-4 text-neutral-900 sm:p-6 lg:p-8">
    <div class="mx-auto max-w-[1600px] space-y-6">
      <!-- Page heading -->
      <header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <div class="flex items-center gap-2 text-xs font-medium text-neutral-500">
            <NuxtLink to="/" class="transition hover:text-neutral-900">
              Dashboard
            </NuxtLink>
            <span>/</span>
            <span class="text-neutral-900">Customers</span>
          </div>

          <h1 class="mt-3 text-2xl font-semibold tracking-tight sm:text-3xl">
            Customers
          </h1>

          <p class="mt-1 text-sm text-neutral-500">
            Manage customer details, contact information and credit terms.
          </p>
        </div>

        <button
          type="button"
          class="inline-flex min-h-11 items-center justify-center gap-2 rounded-lg bg-neutral-900 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-neutral-700"
          @click="openCreateModal"
        >
          <span class="text-lg leading-none">+</span>
          Add customer
        </button>
      </header>

      <!-- Summary cards -->
      <section class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <article class="rounded-xl border border-neutral-200 bg-white p-5">
          <p class="text-sm text-neutral-500">Matching customers</p>
          <p class="mt-3 text-2xl font-semibold tracking-tight">
            {{ loading ? '—' : total.toLocaleString('en-IN') }}
          </p>
        </article>

        <article class="rounded-xl border border-neutral-200 bg-white p-5">
          <p class="text-sm text-neutral-500">Current page</p>
          <p class="mt-3 text-2xl font-semibold tracking-tight">
            {{ loading ? '—' : `${page} / ${totalPages}` }}
          </p>
        </article>

        <article class="rounded-xl border border-neutral-200 bg-white p-5">
          <p class="text-sm text-neutral-500">Records displayed</p>
          <p class="mt-3 text-2xl font-semibold tracking-tight">
            {{ loading ? '—' : customers.length }}
          </p>
        </article>
      </section>

      <!-- Customer directory -->
      <section class="overflow-hidden rounded-xl border border-neutral-200 bg-white">
        <div class="flex flex-col gap-4 border-b border-neutral-200 p-4 sm:p-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 class="font-semibold">Customer directory</h2>
            <p class="mt-1 text-sm text-neutral-500">
              Search and manage registered customers.
            </p>
          </div>

          <div class="flex flex-col gap-3 sm:flex-row">
            <input
              v-model="search"
              type="search"
              placeholder="Search name, code, phone, GSTIN..."
              class="min-h-10 w-full rounded-lg border border-neutral-300 bg-white px-3 text-sm outline-none transition placeholder:text-neutral-400 focus:border-neutral-600 focus:ring-2 focus:ring-neutral-100 sm:w-72"
              @input="handleSearch"
            >

            <select
              v-model="activeFilter"
              class="min-h-10 rounded-lg border border-neutral-300 bg-white px-3 text-sm outline-none focus:border-neutral-600"
              aria-label="Customer status filter"
            >
              <option value="active">Active customers</option>
              <option value="inactive">Inactive customers</option>
              <option value="all">All customers</option>
            </select>

            <button
              type="button"
              class="min-h-10 rounded-lg border border-neutral-300 px-4 text-sm font-medium transition hover:bg-neutral-100 disabled:opacity-50"
              :disabled="loading"
              @click="loadCustomers"
            >
              Refresh
            </button>
          </div>
        </div>

        <!-- Error state -->
        <div
          v-if="pageError"
          role="alert"
          class="m-4 flex flex-col gap-3 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700 sm:flex-row sm:items-center sm:justify-between"
        >
          <p>{{ pageError }}</p>

          <button
            type="button"
            class="shrink-0 font-semibold underline"
            @click="loadCustomers"
          >
            Try again
          </button>
        </div>

        <!-- Customer table -->
        <div class="overflow-x-auto">
          <table class="w-full min-w-[900px] border-collapse text-left">
            <thead class="bg-neutral-50">
              <tr class="border-b border-neutral-200">
                <th class="px-5 py-3.5 text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  Customer
                </th>
                <th class="px-5 py-3.5 text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  Contact
                </th>
                <th class="px-5 py-3.5 text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  GSTIN
                </th>
                <th class="px-5 py-3.5 text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  Location
                </th>
                <th class="px-5 py-3.5 text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  Opening balance
                </th>
                <th class="px-5 py-3.5 text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  Status
                </th>
                <th class="px-5 py-3.5 text-right text-xs font-semibold uppercase tracking-wide text-neutral-500">
                  Actions
                </th>
              </tr>
            </thead>

            <tbody class="divide-y divide-neutral-100">
              <!-- Loading -->
              <tr v-if="loading">
                <td colspan="7" class="px-5 py-16 text-center">
                  <div class="inline-flex items-center gap-3 text-sm text-neutral-500">
                    <span class="h-4 w-4 animate-spin rounded-full border-2 border-neutral-300 border-t-neutral-900" />
                    Loading customers…
                  </div>
                </td>
              </tr>

              <!-- Empty state -->
              <tr v-else-if="!pageError && customers.length === 0">
                <td colspan="7" class="px-5 py-16 text-center">
                  <div class="mx-auto max-w-sm">
                    <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-xl border border-neutral-200 bg-neutral-50 text-xl">
                      —
                    </div>

                    <h3 class="mt-4 font-semibold">
                      {{ search ? 'No matching customers' : 'No customers found' }}
                    </h3>

                    <p class="mt-1 text-sm leading-6 text-neutral-500">
                      {{
                        search
                          ? 'Try a different search term.'
                          : 'Add a customer to begin maintaining your customer directory.'
                      }}
                    </p>

                    <button
                      v-if="!search && activeFilter !== 'inactive'"
                      type="button"
                      class="mt-4 rounded-lg bg-neutral-900 px-4 py-2.5 text-sm font-medium text-white hover:bg-neutral-700"
                      @click="openCreateModal"
                    >
                      Add customer
                    </button>
                  </div>
                </td>
              </tr>

              <!-- Customer records -->
              <tr
                v-for="customer in customers"
                v-else
                :key="customer.id"
                class="transition hover:bg-neutral-50/80"
              >
                <td class="px-5 py-4">
                  <div class="font-medium text-neutral-900">
                    {{ customer.name }}
                  </div>
                  <div class="mt-1 text-xs text-neutral-500">
                    {{ customer.party_code || 'No code' }}
                  </div>
                </td>

                <td class="px-5 py-4">
                  <div class="text-sm text-neutral-800">
                    {{ customer.phone || '—' }}
                  </div>
                  <div class="mt-1 max-w-52 truncate text-xs text-neutral-500">
                    {{ customer.email || customer.contact_person || '—' }}
                  </div>
                </td>

                <td class="px-5 py-4">
                  <span class="text-sm text-neutral-700">
                    {{ customer.gstin || '—' }}
                  </span>
                </td>

                <td class="px-5 py-4">
                  <div class="text-sm text-neutral-800">
                    {{ customer.city || '—' }}
                  </div>
                  <div class="mt-1 text-xs text-neutral-500">
                    {{ customer.state || '' }}
                  </div>
                </td>

                <td class="px-5 py-4">
                  <div class="text-sm font-medium tabular-nums">
                    {{ formatCurrency(customer.opening_balance) }}
                  </div>
                  <div class="mt-1 text-xs text-neutral-500">
                    {{ customer.opening_balance_type || '—' }}
                  </div>
                </td>

                <td class="px-5 py-4">
                  <span
                    class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium"
                    :class="
                      customer.is_active
                        ? 'border-neutral-200 bg-neutral-100 text-neutral-800'
                        : 'border-neutral-200 bg-white text-neutral-500'
                    "
                  >
                    <span
                      class="h-1.5 w-1.5 rounded-full"
                      :class="customer.is_active ? 'bg-neutral-900' : 'bg-neutral-400'"
                    />
                    {{ customer.is_active ? 'Active' : 'Inactive' }}
                  </span>
                </td>

                <td class="px-5 py-4">
                  <div class="flex items-center justify-end gap-3">
                    <button
                      type="button"
                      class="text-sm font-medium text-neutral-700 underline-offset-4 hover:text-black hover:underline"
                      @click="openEditModal(customer)"
                    >
                      Edit
                    </button>

                    <button
                      v-if="customer.is_active"
                      type="button"
                      class="text-sm font-medium text-neutral-500 underline-offset-4 hover:text-red-700 hover:underline"
                      @click="handleDeactivate(customer)"
                    >
                      Deactivate
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Pagination -->
        <footer class="flex flex-col gap-3 border-t border-neutral-200 px-4 py-4 sm:flex-row sm:items-center sm:justify-between sm:px-5">
          <p class="text-sm text-neutral-500">
            <template v-if="total > 0">
              Showing
              <span class="font-medium text-neutral-800">{{ startRecord }}</span>
              to
              <span class="font-medium text-neutral-800">{{ endRecord }}</span>
              of
              <span class="font-medium text-neutral-800">{{ total }}</span>
              customers
            </template>
            <template v-else>
              No records to display
            </template>
          </p>

          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded-lg border border-neutral-300 px-3.5 py-2 text-sm font-medium transition hover:bg-neutral-100 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="loading || page <= 1"
              @click="page--"
            >
              Previous
            </button>

            <span class="min-w-16 text-center text-sm tabular-nums text-neutral-600">
              {{ page }} / {{ totalPages }}
            </span>

            <button
              type="button"
              class="rounded-lg border border-neutral-300 px-3.5 py-2 text-sm font-medium transition hover:bg-neutral-100 disabled:cursor-not-allowed disabled:opacity-40"
              :disabled="loading || page >= totalPages"
              @click="page++"
            >
              Next
            </button>
          </div>
        </footer>
      </section>
    </div>

    <PartyFormModal
      :open="modalOpen"
      :party="selectedParty"
      default-role="CUSTOMER"
      @close="modalOpen = false"
      @saved="handleSaved"
    />
  </main>
</template>
