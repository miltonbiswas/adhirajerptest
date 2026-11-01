<script setup lang="ts">
useHead({
  title: 'Brands | AdhirajPOS',
})
import {
  Search,
  Plus,
  MoreHorizontal,
  Pencil,
  Trash2,
  RefreshCw,
  Tags,
  X,
  ChevronLeft,
  ChevronRight,
  CheckCircle2,
  AlertCircle,
  Building2,
} from 'lucide-vue-next'

interface Brand {
  id: string
  name: string
  manufacturer_id: string | null
  description: string | null
  is_active: boolean
}

interface BrandResponse {
  data: Brand[]
  page: number
  limit: number
  total: number
}

interface Manufacturer {
  id: string
  name: string
  is_active: boolean
}

interface ManufacturerResponse {
  data: Manufacturer[]
  total: number
}

const API = 'http://127.0.0.1:8080/api'

const brands = ref<Brand[]>([])
const manufacturers = ref<Manufacturer[]>([])

const search = ref('')
const page = ref(1)
const limit = ref(25)
const total = ref(0)

const loading = ref(false)
const manufacturersLoading = ref(false)
const saving = ref(false)

const modalOpen = ref(false)
const editingId = ref<string | null>(null)
const menuId = ref<string | null>(null)

const error = ref('')
const success = ref('')
const manufacturerError = ref('')

const deleteTarget = ref<Brand | null>(null)

const form = reactive({
  name: '',
  manufacturer_id: null as string | null,
  description: '',
  is_active: true,
})

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / limit.value)),
)

const selectedManufacturerMissing = computed(() => {
  if (!form.manufacturer_id) return false

  return !manufacturers.value.some(
    manufacturer => manufacturer.id === form.manufacturer_id,
  )
})

let searchTimer: ReturnType<typeof setTimeout> | undefined
let notificationTimer: ReturnType<typeof setTimeout> | undefined

function notify(message: string, isError = false) {
  if (notificationTimer) {
    clearTimeout(notificationTimer)
  }

  error.value = isError ? message : ''
  success.value = isError ? '' : message

  notificationTimer = setTimeout(() => {
    error.value = ''
    success.value = ''
  }, 4000)
}

function getErrorMessage(e: any, fallback: string) {
  return (
    e?.data?.message ||
    e?.data?.error ||
    e?.message ||
    fallback
  )
}

async function loadBrands() {
  loading.value = true
  error.value = ''

  try {
    const response = await $fetch<BrandResponse>(`${API}/brands`, {
      query: {
        search: search.value.trim() || undefined,
        page: page.value,
        limit: limit.value,
      },
    })

    brands.value = response.data
    total.value = response.total

    // Keep the current page valid if records were removed.
    const lastPage = Math.max(1, Math.ceil(response.total / limit.value))

    if (page.value > lastPage) {
      page.value = lastPage
    }
  } catch (e: any) {
    notify(
      getErrorMessage(
        e,
        'Brands load nahi ho sake. Backend/API check karo.',
      ),
      true,
    )
  } finally {
    loading.value = false
  }
}

async function loadManufacturers() {
  manufacturersLoading.value = true
  manufacturerError.value = ''

  try {
    const response = await $fetch<ManufacturerResponse>(
      `${API}/manufacturers`,
      {
        query: {
          limit: 500,
        },
      },
    )

    manufacturers.value = response.data.filter(
      manufacturer => manufacturer.is_active,
    )
  } catch (e: any) {
    manufacturerError.value = getErrorMessage(
      e,
      'Manufacturers load nahi ho sake.',
    )
  } finally {
    manufacturersLoading.value = false
  }
}

function onSearch() {
  if (searchTimer) {
    clearTimeout(searchTimer)
  }

  searchTimer = setTimeout(() => {
    if (page.value !== 1) {
      page.value = 1
    } else {
      loadBrands()
    }
  }, 300)
}

function clearSearch() {
  search.value = ''

  if (searchTimer) {
    clearTimeout(searchTimer)
  }

  if (page.value !== 1) {
    page.value = 1
  } else {
    loadBrands()
  }
}

function resetForm() {
  form.name = ''
  form.manufacturer_id = null
  form.description = ''
  form.is_active = true
}

function openCreate() {
  editingId.value = null
  resetForm()
  menuId.value = null
  modalOpen.value = true
}

function openEdit(brand: Brand) {
  editingId.value = brand.id

  form.name = brand.name
  form.manufacturer_id = brand.manufacturer_id ?? null
  form.description = brand.description ?? ''
  form.is_active = brand.is_active

  menuId.value = null
  modalOpen.value = true
}

function closeModal() {
  if (saving.value) return

  modalOpen.value = false
  editingId.value = null
}

async function saveBrand() {
  const name = form.name.trim()

  if (!name) {
    notify('Brand name required hai.', true)
    return
  }

  if (name.length > 200) {
    notify('Brand name 200 characters se zyada nahi ho sakta.', true)
    return
  }

  saving.value = true

  // Send every editable field on both create and update.
  // This prevents the backend from unintentionally clearing
  // manufacturer_id or description during an update.
  const payload = {
    name,
    manufacturer_id: form.manufacturer_id || null,
    description: form.description.trim() || null,
    is_active: form.is_active,
  }

  try {
    if (editingId.value) {
      await $fetch(`${API}/brands/${encodeURIComponent(editingId.value)}`, {
        method: 'PUT',
        body: payload,
      })

      notify('Brand updated successfully.')
    } else {
      await $fetch(`${API}/brands`, {
        method: 'POST',
        body: payload,
      })

      notify('Brand created successfully.')
      page.value = 1
    }

    modalOpen.value = false
    editingId.value = null

    await loadBrands()
  } catch (e: any) {
    notify(
      getErrorMessage(
        e,
        'Brand save nahi hua. Name, manufacturer aur API check karo.',
      ),
      true,
    )
  } finally {
    saving.value = false
  }
}

async function deleteBrand() {
  if (!deleteTarget.value || saving.value) return

  const brand = deleteTarget.value
  saving.value = true

  try {
    await $fetch(`${API}/brands/${encodeURIComponent(brand.id)}`, {
      method: 'DELETE',
    })

    deleteTarget.value = null
    menuId.value = null

    notify('Brand deleted successfully.')

    if (brands.value.length === 1 && page.value > 1) {
      page.value--
    } else {
      await loadBrands()
    }
  } catch (e: any) {
    notify(
      getErrorMessage(
        e,
        'Brand delete nahi hua. Ho sakta hai koi product is brand se linked ho.',
      ),
      true,
    )
  } finally {
    saving.value = false
  }
}

async function toggleStatus(brand: Brand) {
  if (saving.value) return

  menuId.value = null
  saving.value = true

  // Preserve every existing field when changing only the status.
  const payload = {
    name: brand.name,
    manufacturer_id: brand.manufacturer_id ?? null,
    description: brand.description ?? null,
    is_active: !brand.is_active,
  }

  try {
    await $fetch(`${API}/brands/${encodeURIComponent(brand.id)}`, {
      method: 'PUT',
      body: payload,
    })

    notify(
      payload.is_active
        ? 'Brand activated successfully.'
        : 'Brand deactivated successfully.',
    )

    await loadBrands()
  } catch (e: any) {
    notify(
      getErrorMessage(e, 'Brand status update nahi hua.'),
      true,
    )
  } finally {
    saving.value = false
  }
}

function changePage(nextPage: number) {
  if (nextPage < 1 || nextPage > totalPages.value) return
  if (nextPage === page.value) return

  page.value = nextPage
}

watch(page, () => {
  loadBrands()
})

watch(limit, () => {
  if (page.value !== 1) {
    page.value = 1
  } else {
    loadBrands()
  }
})

onMounted(() => {
  loadBrands()
  loadManufacturers()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  if (notificationTimer) clearTimeout(notificationTimer)
})
</script>

<template>
  <main class="min-h-full bg-[#F8F9FB] p-4 text-[#172033] sm:p-5">
    <!-- Header -->
    <header class="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div>
        <div
          class="mb-1 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.15em] text-[#647793]"
        >
          <Tags class="size-3.5" />
          Utilities / Master data
        </div>

        <h1 class="text-[24px] font-semibold tracking-tight">
          Brands
        </h1>

        <p class="mt-1 text-[12px] text-[#748198]">
          Manage medicine brands and their manufacturers.
        </p>
      </div>

      <button
        class="inline-flex h-9 items-center gap-2 rounded-lg bg-[#087A55] px-3.5 text-[12px] font-medium text-white transition hover:bg-[#066746] disabled:opacity-50"
        @click="openCreate"
      >
        <Plus class="size-4" />
        Add brand
      </button>
    </header>

    <!-- Notifications -->
    <div
      v-if="error || success"
      class="mb-3 flex items-center gap-2 rounded-lg border p-3 text-[12px]"
      :class="
        error
          ? 'border-red-200 bg-red-50 text-red-700'
          : 'border-emerald-200 bg-emerald-50 text-emerald-700'
      "
      role="status"
      aria-live="polite"
    >
      <AlertCircle v-if="error" class="size-4 shrink-0" />
      <CheckCircle2 v-else class="size-4 shrink-0" />

      <span>{{ error || success }}</span>

      <button
        class="ml-auto"
        aria-label="Dismiss notification"
        @click="error = ''; success = ''"
      >
        <X class="size-4" />
      </button>
    </div>

    <!-- Register -->
    <section class="overflow-visible rounded-xl border border-[#E1E7EF] bg-white">
      <div
        class="flex flex-wrap items-center justify-between gap-3 border-b border-[#E8ECF2] px-4 py-3"
      >
        <div>
          <h2 class="text-[13px] font-semibold">
            Brand register
          </h2>

          <p class="mt-0.5 text-[11px] text-[#8490A3]">
            Search, edit and manage existing records.
          </p>
        </div>

        <span
          class="rounded-md border border-[#E1E7EF] px-2.5 py-1.5 text-[11px] text-[#647793]"
        >
          {{ total.toLocaleString('en-IN') }} records
        </span>
      </div>

      <!-- Toolbar -->
      <div
        class="flex flex-wrap items-center gap-2 border-b border-[#E8ECF2] px-3 py-3"
      >
        <label
          class="flex h-9 min-w-[220px] flex-1 items-center gap-2 rounded-lg border border-[#DCE4EF] px-3 focus-within:border-[#92A69D]"
        >
          <Search class="size-4 shrink-0 text-[#8793A6]" />

          <input
            v-model="search"
            class="w-full bg-transparent text-[12px] outline-none placeholder:text-[#9AA5B5]"
            placeholder="Search brand name..."
            @input="onSearch"
          >

          <button
            v-if="search"
            type="button"
            class="text-[#8793A6] hover:text-[#172033]"
            aria-label="Clear search"
            @click="clearSearch"
          >
            <X class="size-3.5" />
          </button>
        </label>

        <button
          class="flex h-9 items-center gap-2 rounded-lg border border-[#DCE4EF] px-3 text-[11px] hover:bg-[#F7F9FC] disabled:opacity-50"
          :disabled="loading"
          @click="loadBrands"
        >
          <RefreshCw
            class="size-3.5"
            :class="{ 'animate-spin': loading }"
          />
          Refresh
        </button>
      </div>

      <!-- Manufacturer load warning -->
      <div
        v-if="manufacturerError"
        class="flex items-start gap-2 border-b border-amber-100 bg-amber-50 px-4 py-2.5 text-[11px] text-amber-800"
      >
        <AlertCircle class="mt-0.5 size-3.5 shrink-0" />

        <div class="flex-1">
          {{ manufacturerError }}
          Brands can still be managed without selecting a manufacturer.
        </div>

        <button
          class="font-semibold underline"
          @click="loadManufacturers"
        >
          Retry
        </button>
      </div>

      <!-- Table -->
      <div class="overflow-x-auto">
        <table class="w-full min-w-[600px] border-collapse text-left">
          <thead class="bg-[#F7F9FC]">
            <tr
              class="border-b border-[#E5EAF1] text-[10px] font-semibold uppercase tracking-wide text-[#687890]"
            >
              <th class="px-4 py-3">#</th>
              <th class="px-4 py-3">Brand name</th>
              <th class="px-4 py-3">Status</th>
              <th class="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>

          <tbody v-if="loading">
            <tr>
              <td
                colspan="4"
                class="px-4 py-12 text-center text-[12px] text-[#748198]"
              >
                Loading brands...
              </td>
            </tr>
          </tbody>

          <tbody v-else-if="brands.length">
            <tr
              v-for="(brand, index) in brands"
              :key="brand.id"
              class="border-b border-[#EDF0F5] last:border-0 hover:bg-[#FAFBFD]"
            >
              <td class="px-4 py-3 text-[11px] text-[#8490A3]">
                {{ (page - 1) * limit + index + 1 }}
              </td>

              <td class="px-4 py-3">
                <div class="flex items-center gap-2.5">
                  <div
                    class="flex size-8 shrink-0 items-center justify-center rounded-lg border border-[#E5EAF1] bg-[#F7F9FC]"
                  >
                    <Tags class="size-4 text-[#63748A]" />
                  </div>

                  <div class="min-w-0">
                    <span class="block text-[12px] font-medium text-[#273449]">
                      {{ brand.name }}
                    </span>

                    <span
                      v-if="brand.description"
                      class="mt-0.5 block max-w-[360px] truncate text-[10px] text-[#8490A3]"
                      :title="brand.description"
                    >
                      {{ brand.description }}
                    </span>
                  </div>
                </div>
              </td>

              <td class="px-4 py-3">
                <span
                  class="inline-flex rounded-full border px-2 py-1 text-[10px] font-medium"
                  :class="
                    brand.is_active
                      ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
                      : 'border-[#E1E7EF] bg-[#F5F6F8] text-[#788497]'
                  "
                >
                  {{ brand.is_active ? 'Active' : 'Inactive' }}
                </span>
              </td>

              <td class="px-4 py-3 text-right">
                <div class="relative inline-block">
                  <button
                    class="flex size-8 items-center justify-center rounded-md text-[#647793] hover:bg-[#EDF2F7]"
                    aria-label="Brand actions"
                    :aria-expanded="menuId === brand.id"
                    @click="menuId = menuId === brand.id ? null : brand.id"
                  >
                    <MoreHorizontal class="size-4" />
                  </button>

                  <div
                    v-if="menuId === brand.id"
                    class="absolute right-0 top-9 z-20 w-48 overflow-hidden rounded-xl border border-[#E1E7EF] bg-white py-1 text-left shadow-[0_8px_28px_rgba(15,23,42,0.12)]"
                  >
                    <p
                      class="border-b border-[#E8ECF2] px-3 py-2 text-[11px] font-semibold text-[#647793]"
                    >
                      Brand actions
                    </p>

                    <button
                      class="flex h-9 w-full items-center gap-2.5 px-3 text-[12px] hover:bg-[#F5F7FA]"
                      @click="openEdit(brand)"
                    >
                      <Pencil class="size-4 text-[#647793]" />
                      Edit brand
                    </button>

                    <button
                      class="flex h-9 w-full items-center gap-2.5 px-3 text-[12px] hover:bg-[#F5F7FA] disabled:opacity-50"
                      :disabled="saving"
                      @click="toggleStatus(brand)"
                    >
                      <CheckCircle2 class="size-4 text-[#647793]" />
                      {{ brand.is_active ? 'Deactivate' : 'Activate' }}
                    </button>

                    <button
                      class="flex h-9 w-full items-center gap-2.5 border-t border-[#E8ECF2] px-3 text-[12px] text-red-600 hover:bg-red-50"
                      @click="deleteTarget = brand; menuId = null"
                    >
                      <Trash2 class="size-4" />
                      Delete brand
                    </button>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>

          <tbody v-else>
            <tr>
              <td colspan="4" class="px-4 py-14 text-center">
                <Tags class="mx-auto mb-3 size-8 text-[#B0BAC8]" />

                <p class="text-[13px] font-medium">
                  No brands found
                </p>

                <p class="mt-1 text-[11px] text-[#8490A3]">
                  Search clear karo ya naya brand add karo.
                </p>

                <button
                  v-if="search"
                  class="mt-3 text-[11px] font-medium text-[#087A55] hover:underline"
                  @click="clearSearch"
                >
                  Clear search
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      <footer
        class="flex flex-wrap items-center justify-between gap-3 border-t border-[#E8ECF2] px-4 py-3"
      >
        <p class="text-[11px] text-[#748198]">
          Showing
          <span class="font-medium text-[#34445D]">
            {{ total ? (page - 1) * limit + 1 : 0 }}–{{ Math.min(page * limit, total) }}
          </span>
          of {{ total.toLocaleString('en-IN') }} brands
        </p>

        <div class="flex items-center gap-2">
          <select
            v-model.number="limit"
            class="h-8 rounded-md border border-[#DCE4EF] bg-white px-2 text-[11px] outline-none"
            aria-label="Rows per page"
          >
            <option :value="25">25 rows</option>
            <option :value="50">50 rows</option>
            <option :value="100">100 rows</option>
          </select>

          <button
            class="flex size-8 items-center justify-center rounded-md border border-[#DCE4EF] disabled:opacity-40"
            aria-label="Previous page"
            :disabled="page <= 1 || loading"
            @click="changePage(page - 1)"
          >
            <ChevronLeft class="size-4" />
          </button>

          <span class="min-w-12 text-center text-[11px] text-[#647793]">
            {{ page }} / {{ totalPages }}
          </span>

          <button
            class="flex size-8 items-center justify-center rounded-md border border-[#DCE4EF] disabled:opacity-40"
            aria-label="Next page"
            :disabled="page >= totalPages || loading"
            @click="changePage(page + 1)"
          >
            <ChevronRight class="size-4" />
          </button>
        </div>
      </footer>
    </section>

    <!-- Create/Edit modal -->
    <div
      v-if="modalOpen"
      class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-[#101828]/35 p-4"
      @click.self="closeModal"
    >
      <section
        class="my-auto w-full max-w-lg rounded-xl border border-[#E1E7EF] bg-white shadow-xl"
        role="dialog"
        aria-modal="true"
        aria-labelledby="brand-modal-title"
      >
        <header
          class="flex items-center justify-between border-b border-[#E8ECF2] px-5 py-4"
        >
          <div>
            <h2 id="brand-modal-title" class="text-[15px] font-semibold">
              {{ editingId ? 'Edit brand' : 'Add brand' }}
            </h2>

            <p class="mt-1 text-[11px] text-[#8490A3]">
              Brand master record
            </p>
          </div>

          <button
            class="flex size-8 items-center justify-center rounded-md hover:bg-[#F2F4F7] disabled:opacity-50"
            aria-label="Close modal"
            :disabled="saving"
            @click="closeModal"
          >
            <X class="size-4" />
          </button>
        </header>

        <form class="space-y-4 p-5" @submit.prevent="saveBrand">
          <!-- Brand name -->
          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Brand name <span class="text-red-500">*</span>
            </span>

            <input
              v-model="form.name"
              autofocus
              required
              maxlength="200"
              placeholder="Enter brand name"
              class="h-10 w-full rounded-lg border border-[#DCE4EF] px-3 text-[12px] outline-none focus:border-[#8DA99A]"
            >
          </label>

          <!-- Manufacturer -->
          <label class="block">
            <span class="mb-1.5 flex items-center gap-1.5 text-[11px] font-medium text-[#46556C]">
              <Building2 class="size-3.5" />
              Manufacturer
              <span class="font-normal text-[#8490A3]">(optional)</span>
            </span>

            <select
              v-model="form.manufacturer_id"
              class="h-10 w-full rounded-lg border border-[#DCE4EF] bg-white px-3 text-[12px] outline-none focus:border-[#8DA99A]"
            >
              <option :value="null">
                No manufacturer selected
              </option>

              <!-- Preserve a linked manufacturer even if it is not in
                   the currently returned list of active manufacturers. -->
              <option
                v-if="selectedManufacturerMissing"
                :value="form.manufacturer_id"
              >
                Existing linked manufacturer
              </option>

              <option
                v-for="manufacturer in manufacturers"
                :key="manufacturer.id"
                :value="manufacturer.id"
              >
                {{ manufacturer.name }}
              </option>
            </select>

            <p
              v-if="manufacturersLoading"
              class="mt-1.5 text-[10px] text-[#8490A3]"
            >
              Loading manufacturers...
            </p>

            <p
              v-else-if="!manufacturers.length && !manufacturerError"
              class="mt-1.5 text-[10px] text-[#8490A3]"
            >
              No active manufacturers available. You can save the brand without one.
            </p>
          </label>

          <!-- Description -->
          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Description
              <span class="font-normal text-[#8490A3]">(optional)</span>
            </span>

            <textarea
              v-model="form.description"
              rows="3"
              maxlength="2000"
              placeholder="Add brand notes or description..."
              class="w-full resize-y rounded-lg border border-[#DCE4EF] px-3 py-2.5 text-[12px] outline-none focus:border-[#8DA99A]"
            />
          </label>

          <!-- Status -->
          <label
            class="flex items-center gap-2.5 rounded-lg border border-[#E1E7EF] p-3"
          >
            <input
              v-model="form.is_active"
              type="checkbox"
              class="size-4 accent-[#087A55]"
            >

            <span>
              <span class="block text-[12px] font-medium">
                Active brand
              </span>

              <span class="mt-0.5 block text-[10px] text-[#8490A3]">
                Inactive brands should not be offered for new use.
              </span>
            </span>
          </label>

          <div
            class="flex justify-end gap-2 border-t border-[#E8ECF2] pt-4"
          >
            <button
              type="button"
              class="h-9 rounded-lg border border-[#DCE4EF] px-4 text-[12px] hover:bg-[#F7F9FC] disabled:opacity-50"
              :disabled="saving"
              @click="closeModal"
            >
              Cancel
            </button>

            <button
              type="submit"
              :disabled="saving"
              class="h-9 rounded-lg bg-[#087A55] px-4 text-[12px] font-medium text-white hover:bg-[#066746] disabled:opacity-50"
            >
              {{
                saving
                  ? 'Saving...'
                  : editingId
                    ? 'Save changes'
                    : 'Create brand'
              }}
            </button>
          </div>
        </form>
      </section>
    </div>

    <!-- Delete confirmation -->
    <div
      v-if="deleteTarget"
      class="fixed inset-0 z-[60] flex items-center justify-center bg-[#101828]/35 p-4"
      @click.self="saving ? null : deleteTarget = null"
    >
      <section
        class="w-full max-w-sm rounded-xl border border-[#E1E7EF] bg-white p-5 shadow-xl"
        role="dialog"
        aria-modal="true"
        aria-labelledby="delete-brand-title"
      >
        <div
          class="flex size-10 items-center justify-center rounded-lg bg-red-50 text-red-600"
        >
          <Trash2 class="size-5" />
        </div>

        <h2 id="delete-brand-title" class="mt-4 text-[15px] font-semibold">
          Delete brand?
        </h2>

        <p class="mt-2 text-[12px] leading-5 text-[#647793]">
          Are you sure you want to delete
          <strong class="text-[#34445D]">{{ deleteTarget.name }}</strong>?
          The backend uses soft delete, so the record is marked inactive
          rather than physically removed.
        </p>

        <div class="mt-5 flex justify-end gap-2">
          <button
            class="h-9 rounded-lg border border-[#DCE4EF] px-3 text-[12px] disabled:opacity-50"
            :disabled="saving"
            @click="deleteTarget = null"
          >
            Cancel
          </button>

          <button
            :disabled="saving"
            class="h-9 rounded-lg bg-red-600 px-3 text-[12px] font-medium text-white disabled:opacity-50"
            @click="deleteBrand"
          >
            {{ saving ? 'Deleting...' : 'Delete brand' }}
          </button>
        </div>
      </section>
    </div>

    <!-- Close menus on outside click -->
    <div
      v-if="menuId"
      class="fixed inset-0 z-10"
      @click="menuId = null"
    />
  </main>
</template>
