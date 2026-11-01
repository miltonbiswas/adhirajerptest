<script setup lang="ts">
import {
  ReceiptText,
  ShieldCheck,
  Database,
  CircleCheck,
  Clock3,
  Plus,
  Search,
  RefreshCw,
  Pencil,
  X,
  ChevronLeft,
  ChevronRight,
  Check,
  AlertCircle,
  LoaderCircle,
  Power,
} from 'lucide-vue-next'

import type { Hsn, HsnInput } from '~/composables/useApi'

useHead({
  title: 'Tax & GST | AdhirajPOS',
})

const {
  getHsn,
  createHsn,
  updateHsn,
  updateHsnStatus,
} = useApi()

// ------------------------------------------------------------
// STATE
// ------------------------------------------------------------

const records = ref<Hsn[]>([])
const search = ref('')
const activeFilter = ref<'all' | 'active' | 'inactive'>('all')

const page = ref(1)
const limit = 50
const total = ref(0)

const loading = ref(false)
const saving = ref(false)
const updatingStatus = ref<string | null>(null)

const errorMessage = ref('')
const successMessage = ref('')

const modalOpen = ref(false)
const editingCode = ref<string | null>(null)

const form = reactive<HsnInput>({
  hsn_code: '',
  description: '',
  gst_rate: 0,
  cgst_rate: 0,
  sgst_rate: 0,
  igst_rate: 0,
})

const formError = ref('')

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / limit)),
)

const activeCount = computed(
  () => records.value.filter((item) => item.is_active).length,
)

const inactiveCount = computed(
  () => records.value.filter((item) => !item.is_active).length,
)

// ------------------------------------------------------------
// DATA
// ------------------------------------------------------------

let searchTimer: ReturnType<typeof setTimeout> | undefined

async function loadHsn() {
  loading.value = true
  errorMessage.value = ''

  try {
    const response = await getHsn({
      search: search.value.trim() || undefined,
      page: page.value,
      limit,
      active:
        activeFilter.value === 'all'
          ? undefined
          : activeFilter.value === 'active',
    })

    records.value = response.data
    total.value = response.total

    // Handle a page that no longer exists after a filter/update.
    if (records.value.length === 0 && page.value > 1) {
      page.value = Math.min(page.value - 1, totalPages.value)
      await loadHsn()
    }
  } catch (error: any) {
    errorMessage.value =
      error?.data?.error ||
      error?.message ||
      'HSN records could not be loaded.'

    records.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  if (searchTimer) clearTimeout(searchTimer)

  searchTimer = setTimeout(() => {
    page.value = 1
    loadHsn()
  }, 300)
}

watch(activeFilter, () => {
  page.value = 1
  loadHsn()
})

onMounted(() => {
  loadHsn()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
})

// ------------------------------------------------------------
// FORM
// ------------------------------------------------------------

function resetForm() {
  form.hsn_code = ''
  form.description = ''
  form.gst_rate = 0
  form.cgst_rate = 0
  form.sgst_rate = 0
  form.igst_rate = 0
  formError.value = ''
}

function openCreateModal() {
  resetForm()
  editingCode.value = null
  modalOpen.value = true
}

function openEditModal(item: Hsn) {
  resetForm()

  editingCode.value = item.hsn_code

  form.hsn_code = item.hsn_code
  form.description = item.description
  form.gst_rate = Number(item.gst_rate)
  form.cgst_rate = Number(item.cgst_rate)
  form.sgst_rate = Number(item.sgst_rate)
  form.igst_rate = Number(item.igst_rate)

  modalOpen.value = true
}

function closeModal() {
  if (saving.value) return

  modalOpen.value = false
  editingCode.value = null
  formError.value = ''
}

function setGstRate(value: number | string) {
  const rate = Number(value)

  if (!Number.isFinite(rate) || rate < 0 || rate > 100) {
    return
  }

  form.gst_rate = rate
  form.cgst_rate = rate / 2
  form.sgst_rate = rate / 2
  form.igst_rate = rate
}

function validateForm(): boolean {
  const code = form.hsn_code.trim()

  if (!code) {
    formError.value = 'HSN code is required.'
    return false
  }

  if (code.length > 8 || !/^[a-zA-Z0-9.]+$/.test(code)) {
    formError.value =
      'HSN code must contain 1–8 letters, digits or dots.'
    return false
  }

  const rates = [
    form.gst_rate,
    form.cgst_rate,
    form.sgst_rate,
    form.igst_rate,
  ]

  if (
    rates.some(
      (rate) =>
        !Number.isFinite(Number(rate)) ||
        Number(rate) < 0 ||
        Number(rate) > 100,
    )
  ) {
    formError.value = 'Tax rates must be between 0 and 100.'
    return false
  }

  if (
    Math.abs(
      Number(form.cgst_rate) +
        Number(form.sgst_rate) -
        Number(form.gst_rate),
    ) > 0.01
  ) {
    formError.value =
      'CGST + SGST should equal the total GST rate.'
    return false
  }

  return true
}

async function saveHsn() {
  formError.value = ''
  successMessage.value = ''

  if (!validateForm()) return

  saving.value = true

  const payload: HsnInput = {
    hsn_code: form.hsn_code.trim(),
    gst_rate: Number(form.gst_rate),
    cgst_rate: Number(form.cgst_rate),
    sgst_rate: Number(form.sgst_rate),
    igst_rate: Number(form.igst_rate),
  }

  try {
    if (editingCode.value) {
      await updateHsn(editingCode.value, payload)
      successMessage.value = 'HSN record updated successfully.'
    } else {
      await createHsn(payload)
      successMessage.value = 'HSN record added successfully.'
      page.value = 1
    }

    modalOpen.value = false
    editingCode.value = null

    await loadHsn()
  } catch (error: any) {
    formError.value =
      error?.data?.error ||
      error?.message ||
      'HSN record could not be saved.'
  } finally {
    saving.value = false
  }
}

// ------------------------------------------------------------
// STATUS
// ------------------------------------------------------------

async function toggleStatus(item: Hsn) {
  const nextStatus = !item.is_active

  const confirmed = window.confirm(
    nextStatus
      ? `Activate HSN ${item.hsn_code}?`
      : `Deactivate HSN ${item.hsn_code}?`,
  )

  if (!confirmed) return

  updatingStatus.value = item.hsn_code
  errorMessage.value = ''
  successMessage.value = ''

  try {
    await updateHsnStatus(item.hsn_code, nextStatus)

    successMessage.value = nextStatus
      ? `HSN ${item.hsn_code} activated.`
      : `HSN ${item.hsn_code} deactivated.`

    await loadHsn()
  } catch (error: any) {
    errorMessage.value =
      error?.data?.error ||
      error?.message ||
      'HSN status could not be updated.'
  } finally {
    updatingStatus.value = null
  }
}

// ------------------------------------------------------------
// PAGINATION / FORMAT
// ------------------------------------------------------------

function changePage(nextPage: number) {
  if (nextPage < 1 || nextPage > totalPages.value) return

  page.value = nextPage
  loadHsn()
}

function formatRate(value: number | string) {
  return `${Number(value).toLocaleString('en-IN', {
    maximumFractionDigits: 2,
  })}%`
}

function formatDate(value: string) {
  if (!value) return '—'

  const date = new Date(value)

  if (Number.isNaN(date.getTime())) return '—'

  return new Intl.DateTimeFormat('en-IN', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  }).format(date)
}
</script>

<template>
  <main class="min-h-full bg-[#F8F9FB] p-4 text-[#172033] sm:p-5">
    <!-- Header -->
    <header class="mb-5 flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
      <div>
        <div class="mb-1 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.15em] text-[#647793]">
          <ReceiptText class="size-3.5" />
          Utilities / Tax configuration
        </div>

        <h1 class="text-[24px] font-semibold tracking-tight">
          Tax & GST
        </h1>

        <p class="mt-1 text-[12px] text-[#748198]">
          Manage HSN classifications and their applicable GST rates.
        </p>
      </div>

      <button
        type="button"
        class="inline-flex h-10 items-center justify-center gap-2 self-start rounded-lg bg-[#172033] px-4 text-[12px] font-semibold text-white transition hover:bg-[#2A354A] disabled:opacity-50 sm:self-auto"
        @click="openCreateModal"
      >
        <Plus class="size-4" />
        Add HSN
      </button>
    </header>

    <!-- Notifications -->
    <div
      v-if="successMessage"
      class="mb-4 flex items-start gap-2 rounded-lg border border-emerald-200 bg-emerald-50 px-3.5 py-3 text-[12px] text-emerald-800"
      role="status"
    >
      <CircleCheck class="mt-0.5 size-4 shrink-0" />
      <p class="flex-1">{{ successMessage }}</p>

      <button
        type="button"
        aria-label="Dismiss success message"
        @click="successMessage = ''"
      >
        <X class="size-4" />
      </button>
    </div>

    <div
      v-if="errorMessage"
      class="mb-4 flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 px-3.5 py-3 text-[12px] text-red-800"
      role="alert"
    >
      <AlertCircle class="mt-0.5 size-4 shrink-0" />
      <p class="flex-1">{{ errorMessage }}</p>

      <button
        type="button"
        aria-label="Dismiss error message"
        @click="errorMessage = ''"
      >
        <X class="size-4" />
      </button>
    </div>

    <!-- Summary cards -->
    <section class="mb-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
      <article class="rounded-xl border border-[#E1E7EF] bg-white p-4">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium text-[#748198]">
            Total HSN records
          </span>
          <Database class="size-4 text-[#647793]" />
        </div>

        <p class="mt-2 text-[25px] font-semibold tracking-tight">
          {{ total.toLocaleString('en-IN') }}
        </p>

        <p class="mt-1 text-[10px] text-[#8490A3]">
          Matching current filters
        </p>
      </article>

      <article class="rounded-xl border border-[#E1E7EF] bg-white p-4">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium text-[#748198]">
            Active on this page
          </span>
          <ShieldCheck class="size-4 text-emerald-700" />
        </div>

        <p class="mt-2 text-[25px] font-semibold tracking-tight">
          {{ activeCount }}
        </p>

        <p class="mt-1 text-[10px] text-[#8490A3]">
          Available for selection
        </p>
      </article>

      <article class="rounded-xl border border-[#E1E7EF] bg-white p-4">
        <div class="flex items-center justify-between">
          <span class="text-[11px] font-medium text-[#748198]">
            Inactive on this page
          </span>
          <Clock3 class="size-4 text-[#647793]" />
        </div>

        <p class="mt-2 text-[25px] font-semibold tracking-tight">
          {{ inactiveCount }}
        </p>

        <p class="mt-1 text-[10px] text-[#8490A3]">
          Currently deactivated
        </p>
      </article>
    </section>

    <!-- Main panel -->
    <section class="overflow-hidden rounded-xl border border-[#E1E7EF] bg-white">
      <div class="border-b border-[#E8ECF2] p-4 sm:p-5">
        <div class="flex flex-col justify-between gap-4 lg:flex-row lg:items-center">
          <div class="flex items-start gap-3">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-lg border border-[#E1E7EF] bg-[#F7F9FC]">
              <ReceiptText class="size-5 text-[#526781]" />
            </div>

            <div>
              <h2 class="text-[14px] font-semibold">
                HSN code master
              </h2>

              <p class="mt-1 text-[11px] leading-5 text-[#748198]">
                HSN descriptions, GST rates and record status from the database.
              </p>
            </div>
          </div>

          <div class="flex flex-col gap-2 sm:flex-row">
            <label class="relative block sm:w-64">
              <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[#8490A3]" />

              <input
                v-model="search"
                type="search"
                placeholder="Search HSN or description..."
                class="h-10 w-full rounded-lg border border-[#E1E7EF] bg-white pl-9 pr-3 text-[12px] outline-none transition placeholder:text-[#9AA4B4] focus:border-[#8996A8]"
                @input="handleSearch"
              />
            </label>

            <select
              v-model="activeFilter"
              aria-label="Filter HSN status"
              class="h-10 rounded-lg border border-[#E1E7EF] bg-white px-3 text-[12px] outline-none focus:border-[#8996A8]"
            >
              <option value="all">All statuses</option>
              <option value="active">Active only</option>
              <option value="inactive">Inactive only</option>
            </select>

            <button
              type="button"
              title="Refresh records"
              aria-label="Refresh records"
              class="inline-flex h-10 items-center justify-center rounded-lg border border-[#E1E7EF] px-3 transition hover:bg-[#F7F9FC] disabled:opacity-50"
              :disabled="loading"
              @click="loadHsn"
            >
              <RefreshCw
                class="size-4 text-[#526781]"
                :class="{ 'animate-spin': loading }"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- Table -->
      <div class="overflow-x-auto">
        <table class="w-full min-w-[850px] border-collapse text-left">
          <thead>
            <tr class="border-b border-[#E8ECF2] bg-[#F8F9FB]">
              <th class="px-4 py-3 text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                HSN code
              </th>

              <th class="px-4 py-3 text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                Description
              </th>

              <th class="px-4 py-3 text-right text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                GST
              </th>

              <th class="px-4 py-3 text-right text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                CGST
              </th>

              <th class="px-4 py-3 text-right text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                SGST
              </th>

              <th class="px-4 py-3 text-right text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                IGST
              </th>

              <th class="px-4 py-3 text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                Status
              </th>

              <th class="px-4 py-3 text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                Updated
              </th>

              <th class="px-4 py-3 text-right text-[10px] font-semibold uppercase tracking-wider text-[#748198]">
                Actions
              </th>
            </tr>
          </thead>

          <tbody>
            <!-- Loading -->
            <tr v-if="loading && records.length === 0">
              <td colspan="9" class="px-4 py-14 text-center">
                <LoaderCircle class="mx-auto size-6 animate-spin text-[#647793]" />
                <p class="mt-3 text-[12px] text-[#748198]">
                  Loading HSN records...
                </p>
              </td>
            </tr>

            <!-- Empty -->
            <tr v-else-if="!loading && records.length === 0">
              <td colspan="9" class="px-4 py-14 text-center">
                <Database class="mx-auto size-7 text-[#A0A9B7]" />

                <p class="mt-3 text-[13px] font-medium">
                  No HSN records found
                </p>

                <p class="mt-1 text-[11px] text-[#748198]">
                  Try a different search or add a new HSN record.
                </p>

                <button
                  type="button"
                  class="mt-4 inline-flex h-9 items-center gap-2 rounded-lg border border-[#E1E7EF] px-3 text-[11px] font-medium hover:bg-[#F7F9FC]"
                  @click="openCreateModal"
                >
                  <Plus class="size-3.5" />
                  Add HSN
                </button>
              </td>
            </tr>

            <!-- Records -->
            <tr
              v-for="item in records"
              :key="item.hsn_code"
              class="border-b border-[#EEF1F5] transition last:border-b-0 hover:bg-[#FAFBFD]"
            >
              <td class="px-4 py-3.5">
                <span class="font-mono text-[12px] font-semibold">
                  {{ item.hsn_code }}
                </span>
              </td>

              <td class="max-w-[240px] px-4 py-3.5">
                <span
                  class="block truncate text-[12px] text-[#526078]"
                  :title="item.description || 'No description'"
                >
                  {{ item.description || '—' }}
                </span>
              </td>

              <td class="px-4 py-3.5 text-right text-[12px] font-semibold">
                {{ formatRate(item.gst_rate) }}
              </td>

              <td class="px-4 py-3.5 text-right text-[12px] text-[#526078]">
                {{ formatRate(item.cgst_rate) }}
              </td>

              <td class="px-4 py-3.5 text-right text-[12px] text-[#526078]">
                {{ formatRate(item.sgst_rate) }}
              </td>

              <td class="px-4 py-3.5 text-right text-[12px] text-[#526078]">
                {{ formatRate(item.igst_rate) }}
              </td>

              <td class="px-4 py-3.5">
                <span
                  class="inline-flex items-center gap-1.5 rounded-full border px-2 py-1 text-[10px] font-medium"
                  :class="
                    item.is_active
                      ? 'border-emerald-200 bg-emerald-50 text-emerald-800'
                      : 'border-[#E1E7EF] bg-[#F7F9FC] text-[#748198]'
                  "
                >
                  <span
                    class="size-1.5 rounded-full"
                    :class="item.is_active ? 'bg-emerald-600' : 'bg-[#A0A9B7]'"
                  />
                  {{ item.is_active ? 'Active' : 'Inactive' }}
                </span>
              </td>

              <td class="whitespace-nowrap px-4 py-3.5 text-[11px] text-[#748198]">
                {{ formatDate(item.updated_at) }}
              </td>

              <td class="px-4 py-3.5">
                <div class="flex items-center justify-end gap-1">
                  <button
                    type="button"
                    title="Edit HSN"
                    :aria-label="`Edit HSN ${item.hsn_code}`"
                    class="flex size-8 items-center justify-center rounded-md border border-transparent text-[#526781] transition hover:border-[#E1E7EF] hover:bg-white"
                    @click="openEditModal(item)"
                  >
                    <Pencil class="size-3.5" />
                  </button>

                  <button
                    type="button"
                    :title="item.is_active ? 'Deactivate HSN' : 'Activate HSN'"
                    :aria-label="`${item.is_active ? 'Deactivate' : 'Activate'} HSN ${item.hsn_code}`"
                    class="flex size-8 items-center justify-center rounded-md border border-transparent text-[#526781] transition hover:border-[#E1E7EF] hover:bg-white disabled:opacity-40"
                    :disabled="updatingStatus === item.hsn_code"
                    @click="toggleStatus(item)"
                  >
                    <LoaderCircle
                      v-if="updatingStatus === item.hsn_code"
                      class="size-3.5 animate-spin"
                    />

                    <Power v-else class="size-3.5" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      <div class="flex flex-col gap-3 border-t border-[#E8ECF2] px-4 py-3.5 sm:flex-row sm:items-center sm:justify-between">
        <p class="text-[11px] text-[#748198]">
          <template v-if="total > 0">
            Showing
            <span class="font-medium text-[#172033]">
              {{ (page - 1) * limit + 1 }}–{{ Math.min(page * limit, total) }}
            </span>
            of
            <span class="font-medium text-[#172033]">
              {{ total.toLocaleString('en-IN') }}
            </span>
            records
          </template>

          <template v-else>
            No records to display
          </template>
        </p>

        <div class="flex items-center gap-2">
          <button
            type="button"
            class="inline-flex h-8 items-center gap-1 rounded-md border border-[#E1E7EF] px-2.5 text-[11px] transition hover:bg-[#F7F9FC] disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="page <= 1 || loading"
            @click="changePage(page - 1)"
          >
            <ChevronLeft class="size-3.5" />
            Previous
          </button>

          <span class="min-w-[76px] text-center text-[11px] text-[#748198]">
            Page {{ page }} of {{ totalPages }}
          </span>

          <button
            type="button"
            class="inline-flex h-8 items-center gap-1 rounded-md border border-[#E1E7EF] px-2.5 text-[11px] transition hover:bg-[#F7F9FC] disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="page >= totalPages || loading"
            @click="changePage(page + 1)"
          >
            Next
            <ChevronRight class="size-3.5" />
          </button>
        </div>
      </div>
    </section>

    <!-- Add / Edit modal -->
    <Teleport to="body">
      <div
        v-if="modalOpen"
        class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-black/40 p-3 backdrop-blur-[2px] sm:p-5"
        @click.self="closeModal"
        @keydown.esc="closeModal"
      >
        <section
          role="dialog"
          aria-modal="true"
          :aria-labelledby="'hsn-modal-title'"
          class="my-auto w-full max-w-xl overflow-hidden rounded-xl border border-[#E1E7EF] bg-white shadow-2xl"
        >
          <!-- Modal header -->
          <header class="flex items-start justify-between border-b border-[#E8ECF2] px-5 py-4">
            <div class="flex items-start gap-3">
              <div class="flex size-9 shrink-0 items-center justify-center rounded-lg border border-[#E1E7EF] bg-[#F7F9FC]">
                <ReceiptText class="size-4 text-[#526781]" />
              </div>

              <div>
                <h2 id="hsn-modal-title" class="text-[15px] font-semibold">
                  {{ editingCode ? 'Edit HSN record' : 'Add new HSN' }}
                </h2>

                <p class="mt-1 text-[11px] text-[#748198]">
                  Enter the HSN classification and applicable GST rates.
                </p>
              </div>
            </div>

            <button
              type="button"
              aria-label="Close modal"
              class="flex size-8 items-center justify-center rounded-md text-[#748198] transition hover:bg-[#F1F3F7] hover:text-[#172033]"
              :disabled="saving"
              @click="closeModal"
            >
              <X class="size-4" />
            </button>
          </header>

          <!-- Modal form -->
          <form class="space-y-4 p-5" @submit.prevent="saveHsn">
            <div
              v-if="formError"
              class="flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 px-3 py-2.5 text-[11px] leading-5 text-red-800"
              role="alert"
            >
              <AlertCircle class="mt-0.5 size-4 shrink-0" />
              {{ formError }}
            </div>

            <div class="grid gap-4 sm:grid-cols-2">
              <label class="block">
                <span class="mb-1.5 block text-[11px] font-medium text-[#526078]">
                  HSN code <span class="text-red-600">*</span>
                </span>

                <input
                  v-model="form.hsn_code"
                  type="text"
                  maxlength="8"
                  required
                  :readonly="!!editingCode"
                  placeholder="e.g. 30049099"
                  autocomplete="off"
                  class="h-10 w-full rounded-lg border border-[#E1E7EF] px-3 font-mono text-[12px] uppercase outline-none transition placeholder:font-sans placeholder:text-[#A0A9B7] focus:border-[#8996A8] read-only:bg-[#F7F9FC] read-only:text-[#748198]"
                />

                <span class="mt-1 block text-[10px] text-[#8490A3]">
                  Maximum 8 characters.
                </span>
              </label>

              <label class="block">
                <span class="mb-1.5 block text-[11px] font-medium text-[#526078]">
                  GST rate (%) <span class="text-red-600">*</span>
                </span>

                <select
                  :value="form.gst_rate"
                  class="h-10 w-full rounded-lg border border-[#E1E7EF] bg-white px-3 text-[12px] outline-none transition focus:border-[#8996A8]"
                  @change="setGstRate(($event.target as HTMLSelectElement).value)"
                >
                  <option :value="0">0%</option>
                  <option :value="5">5%</option>
                  <option :value="12">12%</option>
                  <option :value="18">18%</option>
                  <option :value="28">28%</option>
                </select>

                <span class="mt-1 block text-[10px] text-[#8490A3]">
                  Selecting a rate splits it equally into CGST and SGST.
                </span>
              </label>
            </div>

            <label class="block">
              <span class="mb-1.5 block text-[11px] font-medium text-[#526078]">
                Description
              </span>

              <textarea
                v-model="form.description"
                rows="2"
                maxlength="2000"
                placeholder="Enter HSN classification description..."
                class="w-full resize-y rounded-lg border border-[#E1E7EF] px-3 py-2.5 text-[12px] leading-5 outline-none transition placeholder:text-[#A0A9B7] focus:border-[#8996A8]"
              />
            </label>

            <div class="rounded-lg border border-[#E1E7EF] bg-[#F8F9FB] p-3.5">
              <div class="mb-3 flex items-center justify-between">
                <h3 class="text-[12px] font-semibold">
                  Tax breakdown
                </h3>

                <span class="text-[10px] text-[#748198]">
                  Percentage (%)
                </span>
              </div>

              <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
                <label class="block">
                  <span class="mb-1.5 block text-[10px] font-medium text-[#748198]">
                    Total GST
                  </span>

                  <input
                    v-model.number="form.gst_rate"
                    type="number"
                    min="0"
                    max="100"
                    step="0.01"
                    required
                    class="h-9 w-full rounded-md border border-[#E1E7EF] bg-white px-2.5 text-[12px] outline-none focus:border-[#8996A8]"
                  />
                </label>

                <label class="block">
                  <span class="mb-1.5 block text-[10px] font-medium text-[#748198]">
                    CGST
                  </span>

                  <input
                    v-model.number="form.cgst_rate"
                    type="number"
                    min="0"
                    max="100"
                    step="0.01"
                    required
                    class="h-9 w-full rounded-md border border-[#E1E7EF] bg-white px-2.5 text-[12px] outline-none focus:border-[#8996A8]"
                  />
                </label>

                <label class="block">
                  <span class="mb-1.5 block text-[10px] font-medium text-[#748198]">
                    SGST
                  </span>

                  <input
                    v-model.number="form.sgst_rate"
                    type="number"
                    min="0"
                    max="100"
                    step="0.01"
                    required
                    class="h-9 w-full rounded-md border border-[#E1E7EF] bg-white px-2.5 text-[12px] outline-none focus:border-[#8996A8]"
                  />
                </label>

                <label class="block">
                  <span class="mb-1.5 block text-[10px] font-medium text-[#748198]">
                    IGST
                  </span>

                  <input
                    v-model.number="form.igst_rate"
                    type="number"
                    min="0"
                    max="100"
                    step="0.01"
                    required
                    class="h-9 w-full rounded-md border border-[#E1E7EF] bg-white px-2.5 text-[12px] outline-none focus:border-[#8996A8]"
                  />
                </label>
              </div>

              <p class="mt-3 text-[10px] leading-4 text-[#748198]">
                CGST and SGST usually apply to intra-state supplies; IGST
                generally applies to inter-state supplies. Verify the
                applicable classification and rate before saving.
              </p>
            </div>

            <footer class="flex flex-col-reverse gap-2 border-t border-[#E8ECF2] pt-4 sm:flex-row sm:justify-end">
              <button
                type="button"
                class="inline-flex h-10 items-center justify-center rounded-lg border border-[#E1E7EF] px-4 text-[12px] font-medium transition hover:bg-[#F7F9FC] disabled:opacity-50"
                :disabled="saving"
                @click="closeModal"
              >
                Cancel
              </button>

              <button
                type="submit"
                class="inline-flex h-10 items-center justify-center gap-2 rounded-lg bg-[#172033] px-4 text-[12px] font-semibold text-white transition hover:bg-[#2A354A] disabled:cursor-not-allowed disabled:opacity-60"
                :disabled="saving"
              >
                <LoaderCircle
                  v-if="saving"
                  class="size-4 animate-spin"
                />

                <Check v-else class="size-4" />

                {{
                  saving
                    ? 'Saving...'
                    : editingCode
                      ? 'Save changes'
                      : 'Create HSN'
                }}
              </button>
            </footer>
          </form>
        </section>
      </div>
    </Teleport>
  </main>
</template>
