<script setup lang="ts">
useHead({
  title: 'Compositions | AdhirajPOS',
})
import {
  Search, Plus, MoreHorizontal, Pencil, Trash2,
  RefreshCw, FlaskConical, X, ChevronLeft, ChevronRight,
  CheckCircle2, AlertCircle
} from 'lucide-vue-next'

interface Composition {
  id: string
  name: string
  is_active: boolean
}

interface CompositionResponse {
  data: Composition[]
  page: number
  limit: number
  total: number
}

const API = 'http://127.0.0.1:8080/api'

const items = ref<Composition[]>([])
const search = ref('')
const page = ref(1)
const limit = ref(25)
const total = ref(0)

const loading = ref(false)
const saving = ref(false)
const modalOpen = ref(false)
const editingId = ref<string | null>(null)
const menuId = ref<string | null>(null)
const error = ref('')
const success = ref('')
const deleteTarget = ref<Composition | null>(null)

const form = reactive({
  name: '',
  is_active: true,
})

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / limit.value))
)

let searchTimer: ReturnType<typeof setTimeout> | undefined

function notify(message: string, isError = false) {
  error.value = isError ? message : ''
  success.value = isError ? '' : message

  setTimeout(() => {
    error.value = ''
    success.value = ''
  }, 4000)
}

async function loadCompositions() {
  loading.value = true
  error.value = ''

  try {
    const response = await $fetch<CompositionResponse>(`${API}/compositions`, {
      query: {
        search: search.value.trim() || undefined,
        page: page.value,
        limit: limit.value,
      },
    })

    items.value = response.data
    total.value = response.total
  } catch (e: any) {
    notify(
      e?.data?.message ||
      e?.message ||
      'Compositions load nahi ho sake. Backend/API check karo.',
      true,
    )
  } finally {
    loading.value = false
  }
}

function onSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    page.value = 1
    loadCompositions()
  }, 300)
}

function openCreate() {
  editingId.value = null
  form.name = ''
  form.is_active = true
  modalOpen.value = true
}

function openEdit(item: Composition) {
  editingId.value = item.id
  form.name = item.name
  form.is_active = item.is_active
  menuId.value = null
  modalOpen.value = true
}

async function saveComposition() {
  const name = form.name.trim()

  if (!name) {
    notify('Composition name required hai.', true)
    return
  }

  saving.value = true

  try {
    if (editingId.value) {
      await $fetch(
        `${API}/compositions/${encodeURIComponent(editingId.value)}`,
        {
          method: 'PUT',
          body: { name, is_active: form.is_active },
        },
      )

      notify('Composition updated.')
    } else {
      await $fetch(`${API}/compositions`, {
        method: 'POST',
        body: { name, is_active: form.is_active },
      })

      notify('Composition created.')
    }

    modalOpen.value = false
    await loadCompositions()
  } catch (e: any) {
    notify(
      e?.data?.message ||
      e?.data?.error ||
      e?.message ||
      'Save failed. Backend CRUD endpoint verify karo.',
      true,
    )
  } finally {
    saving.value = false
  }
}

async function deleteComposition() {
  if (!deleteTarget.value) return

  saving.value = true

  try {
    await $fetch(
      `${API}/compositions/${encodeURIComponent(deleteTarget.value.id)}`,
      { method: 'DELETE' },
    )

    deleteTarget.value = null
    menuId.value = null
    notify('Composition deleted.')

    if (items.value.length === 1 && page.value > 1) page.value--

    await loadCompositions()
  } catch (e: any) {
    notify(
      e?.data?.message ||
      e?.data?.error ||
      e?.message ||
      'Delete failed. Composition product se linked ho sakti hai.',
      true,
    )
  } finally {
    saving.value = false
  }
}

async function toggleStatus(item: Composition) {
  menuId.value = null

  try {
    await $fetch(
      `${API}/compositions/${encodeURIComponent(item.id)}`,
      {
        method: 'PUT',
        body: {
          name: item.name,
          is_active: !item.is_active,
        },
      },
    )

    notify('Composition status updated.')
    await loadCompositions()
  } catch (e: any) {
    notify(
      e?.data?.message ||
      e?.message ||
      'Status update failed.',
      true,
    )
  }
}

watch(limit, () => {
  page.value = 1
  loadCompositions()
})

watch(page, loadCompositions)

onMounted(loadCompositions)

onBeforeUnmount(() => clearTimeout(searchTimer))
</script>

<template>
  <main class="min-h-full bg-[#F8F9FB] p-4 text-[#172033] sm:p-5">
    <header class="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div>
        <div class="mb-1 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.15em] text-[#647793]">
          <FlaskConical class="size-3.5" />
          Utilities / Master data
        </div>

        <h1 class="text-[24px] font-semibold tracking-tight">Compositions</h1>

        <p class="mt-1 text-[12px] text-[#748198]">
          Manage generic names and active pharmaceutical ingredients.
        </p>
      </div>

      <button
        class="inline-flex h-9 items-center gap-2 rounded-lg bg-[#087A55] px-3.5 text-[12px] font-medium text-white hover:bg-[#066746]"
        @click="openCreate"
      >
        <Plus class="size-4" />
        Add composition
      </button>
    </header>

    <div
      v-if="error || success"
      class="mb-3 flex items-center gap-2 rounded-lg border p-3 text-[12px]"
      :class="error
        ? 'border-red-200 bg-red-50 text-red-700'
        : 'border-emerald-200 bg-emerald-50 text-emerald-700'"
    >
      <AlertCircle v-if="error" class="size-4" />
      <CheckCircle2 v-else class="size-4" />
      {{ error || success }}
      <button class="ml-auto" @click="error = ''; success = ''">
        <X class="size-4" />
      </button>
    </div>

    <section class="overflow-visible rounded-xl border border-[#E1E7EF] bg-white">
      <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[#E8ECF2] px-4 py-3">
        <div>
          <h2 class="text-[13px] font-semibold">Composition register</h2>
          <p class="mt-0.5 text-[11px] text-[#8490A3]">
            Search, edit and manage existing records.
          </p>
        </div>

        <span class="rounded-md border border-[#E1E7EF] px-2.5 py-1.5 text-[11px] text-[#647793]">
          {{ total.toLocaleString('en-IN') }} records
        </span>
      </div>

      <div class="flex flex-wrap items-center gap-2 border-b border-[#E8ECF2] px-3 py-3">
        <label class="flex h-9 min-w-[220px] flex-1 items-center gap-2 rounded-lg border border-[#DCE4EF] px-3 focus-within:border-[#92A69D]">
          <Search class="size-4 shrink-0 text-[#8793A6]" />

          <input
            v-model="search"
            class="w-full bg-transparent text-[12px] outline-none placeholder:text-[#9AA5B5]"
            placeholder="Search composition or generic name..."
            @input="onSearch"
          >

          <button
            v-if="search"
            class="text-[#8793A6]"
            @click="search = ''; onSearch()"
          >
            <X class="size-3.5" />
          </button>
        </label>

        <button
          class="flex h-9 items-center gap-2 rounded-lg border border-[#DCE4EF] px-3 text-[11px] hover:bg-[#F7F9FC] disabled:opacity-50"
          :disabled="loading"
          @click="loadCompositions"
        >
          <RefreshCw class="size-3.5" :class="{ 'animate-spin': loading }" />
          Refresh
        </button>
      </div>

      <div class="overflow-x-auto">
        <table class="w-full min-w-[600px] border-collapse text-left">
          <thead class="bg-[#F7F9FC]">
            <tr class="border-b border-[#E5EAF1] text-[10px] font-semibold uppercase tracking-wide text-[#687890]">
              <th class="px-4 py-3">#</th>
              <th class="px-4 py-3">Composition / generic name</th>
              <th class="px-4 py-3">Status</th>
              <th class="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>

          <tbody v-if="loading">
            <tr>
              <td colspan="4" class="px-4 py-12 text-center text-[12px] text-[#748198]">
                Loading compositions...
              </td>
            </tr>
          </tbody>

          <tbody v-else-if="items.length">
            <tr
              v-for="(item, index) in items"
              :key="item.id"
              class="border-b border-[#EDF0F5] last:border-0 hover:bg-[#FAFBFD]"
            >
              <td class="px-4 py-3 text-[11px] text-[#8490A3]">
                {{ (page - 1) * limit + index + 1 }}
              </td>

              <td class="px-4 py-3">
                <div class="flex items-center gap-2.5">
                  <div class="flex size-8 shrink-0 items-center justify-center rounded-lg border border-[#E5EAF1] bg-[#F7F9FC]">
                    <FlaskConical class="size-4 text-[#63748A]" />
                  </div>

                  <span class="text-[12px] font-medium text-[#273449]">
                    {{ item.name }}
                  </span>
                </div>
              </td>

              <td class="px-4 py-3">
                <span
                  class="inline-flex rounded-full border px-2 py-1 text-[10px] font-medium"
                  :class="item.is_active
                    ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
                    : 'border-[#E1E7EF] bg-[#F5F6F8] text-[#788497]'"
                >
                  {{ item.is_active ? 'Active' : 'Inactive' }}
                </span>
              </td>

              <td class="px-4 py-3 text-right">
                <div class="relative inline-block">
                  <button
                    class="flex size-8 items-center justify-center rounded-md text-[#647793] hover:bg-[#EDF2F7]"
                    aria-label="Composition actions"
                    @click="menuId = menuId === item.id ? null : item.id"
                  >
                    <MoreHorizontal class="size-4" />
                  </button>

                  <div
                    v-if="menuId === item.id"
                    class="absolute right-0 top-9 z-20 w-48 overflow-hidden rounded-xl border border-[#E1E7EF] bg-white py-1 text-left shadow-[0_8px_28px_rgba(15,23,42,0.12)]"
                  >
                    <p class="border-b border-[#E8ECF2] px-3 py-2 text-[11px] font-semibold text-[#647793]">
                      Composition actions
                    </p>

                    <button
                      class="flex h-9 w-full items-center gap-2.5 px-3 text-[12px] hover:bg-[#F5F7FA]"
                      @click="openEdit(item)"
                    >
                      <Pencil class="size-4 text-[#647793]" />
                      Edit composition
                    </button>

                    <button
                      class="flex h-9 w-full items-center gap-2.5 px-3 text-[12px] hover:bg-[#F5F7FA]"
                      @click="toggleStatus(item)"
                    >
                      <CheckCircle2 class="size-4 text-[#647793]" />
                      {{ item.is_active ? 'Deactivate' : 'Activate' }}
                    </button>

                    <button
                      class="flex h-9 w-full items-center gap-2.5 border-t border-[#E8ECF2] px-3 text-[12px] text-red-600 hover:bg-red-50"
                      @click="deleteTarget = item; menuId = null"
                    >
                      <Trash2 class="size-4" />
                      Delete composition
                    </button>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>

          <tbody v-else>
            <tr>
              <td colspan="4" class="px-4 py-14 text-center">
                <FlaskConical class="mx-auto mb-3 size-8 text-[#B0BAC8]" />
                <p class="text-[13px] font-medium">No compositions found</p>
                <p class="mt-1 text-[11px] text-[#8490A3]">
                  Search clear karo ya composition add karo.
                </p>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <footer class="flex flex-wrap items-center justify-between gap-3 border-t border-[#E8ECF2] px-4 py-3">
        <p class="text-[11px] text-[#748198]">
          Showing
          <span class="font-medium text-[#34445D]">
            {{ total ? (page - 1) * limit + 1 : 0 }}–{{ Math.min(page * limit, total) }}
          </span>
          of {{ total.toLocaleString('en-IN') }} compositions
        </p>

        <div class="flex items-center gap-2">
          <select
            v-model.number="limit"
            class="h-8 rounded-md border border-[#DCE4EF] bg-white px-2 text-[11px] outline-none"
          >
            <option :value="25">25 rows</option>
            <option :value="50">50 rows</option>
            <option :value="100">100 rows</option>
          </select>

          <button
            class="flex size-8 items-center justify-center rounded-md border border-[#DCE4EF] disabled:opacity-40"
            :disabled="page <= 1"
            @click="page--"
          >
            <ChevronLeft class="size-4" />
          </button>

          <span class="min-w-12 text-center text-[11px] text-[#647793]">
            {{ page }} / {{ totalPages }}
          </span>

          <button
            class="flex size-8 items-center justify-center rounded-md border border-[#DCE4EF] disabled:opacity-40"
            :disabled="page >= totalPages"
            @click="page++"
          >
            <ChevronRight class="size-4" />
          </button>
        </div>
      </footer>
    </section>

    <!-- Create/Edit modal -->
    <div
      v-if="modalOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-[#101828]/35 p-4"
      @click.self="modalOpen = false"
    >
      <section class="w-full max-w-md rounded-xl border border-[#E1E7EF] bg-white shadow-xl">
        <header class="flex items-center justify-between border-b border-[#E8ECF2] px-5 py-4">
          <div>
            <h2 class="text-[15px] font-semibold">
              {{ editingId ? 'Edit composition' : 'Add composition' }}
            </h2>
            <p class="mt-1 text-[11px] text-[#8490A3]">
              Composition master record
            </p>
          </div>

          <button
            class="flex size-8 items-center justify-center rounded-md hover:bg-[#F2F4F7]"
            @click="modalOpen = false"
          >
            <X class="size-4" />
          </button>
        </header>

        <form class="space-y-4 p-5" @submit.prevent="saveComposition">
          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Composition name <span class="text-red-500">*</span>
            </span>

            <input
              v-model="form.name"
              autofocus
              required
              maxlength="250"
              placeholder="e.g. Paracetamol 500 mg"
              class="h-10 w-full rounded-lg border border-[#DCE4EF] px-3 text-[12px] outline-none focus:border-[#8DA99A]"
            >
          </label>

          <label class="flex items-center gap-2.5 rounded-lg border border-[#E1E7EF] p-3">
            <input
              v-model="form.is_active"
              type="checkbox"
              class="size-4 accent-[#087A55]"
            >
            <span>
              <span class="block text-[12px] font-medium">Active composition</span>
              <span class="mt-0.5 block text-[10px] text-[#8490A3]">
                Keep this record available for use.
              </span>
            </span>
          </label>

          <div class="flex justify-end gap-2 border-t border-[#E8ECF2] pt-4">
            <button
              type="button"
              class="h-9 rounded-lg border border-[#DCE4EF] px-4 text-[12px]"
              @click="modalOpen = false"
            >
              Cancel
            </button>

            <button
              type="submit"
              :disabled="saving"
              class="h-9 rounded-lg bg-[#087A55] px-4 text-[12px] font-medium text-white disabled:opacity-50"
            >
              {{ saving ? 'Saving...' : editingId ? 'Save changes' : 'Create composition' }}
            </button>
          </div>
        </form>
      </section>
    </div>

    <!-- Delete confirmation -->
    <div
      v-if="deleteTarget"
      class="fixed inset-0 z-[60] flex items-center justify-center bg-[#101828]/35 p-4"
      @click.self="deleteTarget = null"
    >
      <section class="w-full max-w-sm rounded-xl border border-[#E1E7EF] bg-white p-5 shadow-xl">
        <div class="flex size-10 items-center justify-center rounded-lg bg-red-50 text-red-600">
          <Trash2 class="size-5" />
        </div>

        <h2 class="mt-4 text-[15px] font-semibold">Delete composition?</h2>

        <p class="mt-2 text-[12px] leading-5 text-[#647793]">
          Delete
          <strong class="text-[#34445D]">{{ deleteTarget.name }}</strong>?
          Linked products aur historical records ko backend safely protect karna chahiye.
        </p>

        <div class="mt-5 flex justify-end gap-2">
          <button
            class="h-9 rounded-lg border border-[#DCE4EF] px-3 text-[12px]"
            @click="deleteTarget = null"
          >
            Cancel
          </button>

          <button
            :disabled="saving"
            class="h-9 rounded-lg bg-red-600 px-3 text-[12px] font-medium text-white disabled:opacity-50"
            @click="deleteComposition"
          >
            {{ saving ? 'Deleting...' : 'Delete composition' }}
          </button>
        </div>
      </section>
    </div>

    <div
      v-if="menuId"
      class="fixed inset-0 z-10"
      @click="menuId = null"
    />
  </main>
</template>