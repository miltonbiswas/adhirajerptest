<script setup lang="ts">
useHead({ title: 'Products' })
import ProductFormModal from '~/components/products/ProductFormModal.vue'
import {
  Search,
  Plus,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  MoreHorizontal,
  Eye,
  Pencil,
  Copy,
  Trash2,
  X,
  Package,
  Filter,
  Check,
  AlertCircle,
  LoaderCircle,
  FlaskConical,
  Barcode,
} from 'lucide-vue-next'

import { computed, onMounted, onBeforeUnmount, reactive, ref, watch } from 'vue'
import { useApi, type Product, type ProductInput, type Brand, type Composition, type MedicineSchedule } from '~/composables/useApi'

const api = useApi()

const products = ref<Product[]>([])
const brands = ref<Brand[]>([])
const compositions = ref<Composition[]>([])
const brandSearch = ref('')
const compositionSearch = ref('')
const brandDropdownOpen = ref(false)
const compositionDropdownOpen = ref(false)
const newBrandName = ref('')
const newCompositionName = ref('')
const addingBrand = ref(false)
const addingComposition = ref(false)
const masterLoading = ref(false)
const unitsPerPack = ref(1)
const total = ref(0)
const page = ref(1)
const limit = ref(50)

const search = ref('')
const statusFilter = ref('all')
const formFilter = ref('all')
const prescriptionFilter = ref('all')

const loading = ref(false)
const saving = ref(false)
const errorMessage = ref('')
const successMessage = ref('')

const activeMenu = ref<string | null>(null)
const selectedProduct = ref<Product | null>(null)
const dialog = ref<'create' | 'edit' | 'view' | 'delete' | null>(null)

const form = reactive<ProductInput>({
  sku: '',
  name: '',
  generic_name: '',
  brand_name: '',
  brand_id: null,
  composition_ids: [],
  dosage_form: '',
  strength: '',
  pack_size: '',
  unit: 'PCS',
  hsn_code: '',
  gst_rate: 0,
  barcode: '',
  is_prescription_required: false,
  is_active: true,
  schedule_class: 'OTC',
  is_narcotic: false,
  units_per_pack: 1,
})

const filteredBrands = computed(() => {
  const q = brandSearch.value.trim().toLowerCase()
  return brands.value.filter(item => !q || item.name.toLowerCase().includes(q))
})
const filteredCompositions = computed(() => {
  const q = compositionSearch.value.trim().toLowerCase()
  return compositions.value.filter(item => !q || item.name.toLowerCase().includes(q))
})
const selectedCompositionName = computed(() => {
  const id = (form.composition_ids || [])[0]
  return compositions.value.find(item => item.id === id)?.name || selectedProduct.value?.composition_names?.[0] || ''
})

function selectBrand(id: string | null) {
  const brand = brands.value.find(item => item.id === id)
  form.brand_id = id
  form.brand_name = brand?.name || ''
  brandSearch.value = ''
  brandDropdownOpen.value = false
}

function selectComposition(id: string | null) {
  const composition = compositions.value.find(item => item.id === id)
  form.composition_ids = id ? [id] : []
  compositionSearch.value = ''
  compositionDropdownOpen.value = false
  if (composition) showSuccess('Composition selected.')
}

const filteredProducts = computed(() => {
  return products.value.filter((product) => {
    if (statusFilter.value === 'active' && !product.is_active) return false
    if (statusFilter.value === 'inactive' && product.is_active) return false

    if (
      formFilter.value !== 'all' &&
      (product.dosage_form || '').toLowerCase() !== formFilter.value.toLowerCase()
    ) {
      return false
    }

    if (
      prescriptionFilter.value === 'required' &&
      !product.is_prescription_required
    ) {
      return false
    }

    if (
      prescriptionFilter.value === 'not_required' &&
      product.is_prescription_required
    ) {
      return false
    }

    return true
  })
})

const totalPages = computed(() => Math.max(1, Math.ceil(total.value / limit.value)))
const startRecord = computed(() => total.value === 0 ? 0 : (page.value - 1) * limit.value + 1)
const endRecord = computed(() => Math.min(page.value * limit.value, total.value))

const activeOnPage = computed(() =>
  products.value.filter(product => product.is_active).length,
)

const inactiveOnPage = computed(() =>
  products.value.filter(product => !product.is_active).length,
)

const dosageForms = computed(() => {
  const forms = products.value
    .map(product => product.dosage_form?.trim())
    .filter((value): value is string => Boolean(value))

  return [...new Set(forms)].sort((a, b) => a.localeCompare(b))
})

let searchTimer: ReturnType<typeof setTimeout> | undefined
let messageTimer: ReturnType<typeof setTimeout> | undefined

async function loadProducts() {
  loading.value = true
  errorMessage.value = ''
  activeMenu.value = null

  try {
    const result = await api.getProducts({
      search: search.value.trim() || undefined,
      page: page.value,
      limit: limit.value,
    })

    products.value = result.data
    total.value = result.total
  } catch (error: unknown) {
    products.value = []
    total.value = 0

    const apiError = error as {
      data?: { error?: string; message?: string }
      message?: string
    }

    errorMessage.value =
      apiError.data?.error ||
      apiError.data?.message ||
      apiError.message ||
      'Products load nahi ho paaye. Backend server check karo.'
  } finally {
    loading.value = false
  }
}

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)

  searchTimer = setTimeout(() => {
    page.value = 1
    loadProducts()
  }, 350)
}

function changePage(nextPage: number) {
  if (nextPage < 1 || nextPage > totalPages.value || loading.value) return
  page.value = nextPage
  loadProducts()
}

function changeLimit(event: Event) {
  const target = event.target as HTMLSelectElement
  limit.value = Number(target.value)
  page.value = 1
  loadProducts()
}

function resetFilters() {
  search.value = ''
  statusFilter.value = 'all'
  formFilter.value = 'all'
  prescriptionFilter.value = 'all'
  page.value = 1
  loadProducts()
}

function clearMessages() {
  successMessage.value = ''
  errorMessage.value = ''
  if (messageTimer) clearTimeout(messageTimer)
}

function showSuccess(message: string) {
  errorMessage.value = ''
  successMessage.value = message

  if (messageTimer) clearTimeout(messageTimer)
  messageTimer = setTimeout(() => {
    successMessage.value = ''
  }, 3500)
}

async function loadMasters() {
  masterLoading.value = true
  try {
    const [brandResult, compositionResult] = await Promise.all([
      api.getBrands({ search: brandSearch.value.trim() || undefined, page: 1, limit: 100 }),
      api.getCompositions({ search: compositionSearch.value.trim() || undefined, page: 1, limit: 100 }),
    ])
    brands.value = brandResult.data.filter(item => item.is_active)
    compositions.value = compositionResult.data.filter(item => item.is_active)
  } catch (error) {
    const apiError = error as { data?: { error?: string; message?: string }; message?: string }
    errorMessage.value = apiError.data?.error || apiError.data?.message || apiError.message || 'Brand/composition master load nahi ho paaye.'
  } finally {
    masterLoading.value = false
  }
}

async function addBrandInline() {
  const name = newBrandName.value.trim()
  if (!name || addingBrand.value) return
  addingBrand.value = true
  try {
    const brand = await api.createBrand({ name, is_active: true })
    brands.value = [...brands.value.filter(item => item.id !== brand.id), brand].sort((a, b) => a.name.localeCompare(b.name))
    form.brand_id = brand.id
    form.brand_name = brand.name
    newBrandName.value = ''
    showSuccess('Brand master mein add ho gaya.')
  } catch (error) {
    const apiError = error as { data?: { error?: string; message?: string }; message?: string }
    errorMessage.value = apiError.data?.error || apiError.data?.message || apiError.message || 'Brand add nahi ho paaya.'
  } finally {
    addingBrand.value = false
  }
}

async function addCompositionInline() {
  const name = newCompositionName.value.trim()
  if (!name || addingComposition.value) return

  addingComposition.value = true

  try {
    const composition = await api.createComposition({
      name,
      is_active: true,
    })

    compositions.value = [
      ...compositions.value.filter(item => item.id !== composition.id),
      composition,
    ].sort((a, b) => a.name.localeCompare(b.name))

    // One composition per product.
    form.composition_ids = [composition.id]

    newCompositionName.value = ''
    showSuccess('Composition master mein add ho gayi.')
  } catch (error: unknown) {
    const apiError = error as {
      data?: { error?: string; message?: string }
      message?: string
    }

    errorMessage.value =
      apiError.data?.error ||
      apiError.data?.message ||
      apiError.message ||
      'Composition add nahi ho paayi.'
  } finally {
    addingComposition.value = false
  }
}

function compositionSelected(id: string) {
  return (form.composition_ids || [])[0] === id
}

function toggleComposition(id: string, checked: boolean) {
  form.composition_ids = checked ? [id] : []
}

function resetForm() {
  Object.assign(form, {
    sku: '',
    name: '',
    generic_name: '',
    brand_name: '',
    brand_id: null,
    composition_ids: [],
    dosage_form: '',
    strength: '',
    pack_size: '',
    unit: 'PCS',
    hsn_code: '',
    gst_rate: 0,
    barcode: '',
    is_prescription_required: false,
    is_active: true,
    schedule_class: 'OTC',
    is_narcotic: false,
    units_per_pack: 1,
  })
  unitsPerPack.value = 1
  brandSearch.value = ''
  compositionSearch.value = ''
  newBrandName.value = ''
  newCompositionName.value = ''
  loadMasters()
}

function openCreate() {
  resetForm()
  selectedProduct.value = null
  activeMenu.value = null
  clearMessages()
  dialog.value = 'create'
}

function openEdit(product: Product) {
  selectedProduct.value = product

  Object.assign(form, {
    sku: product.sku,
    name: product.name,
    generic_name: product.generic_name || '',
    brand_name: product.brand_name || '',
    brand_id: product.brand_id || null,
    composition_ids: [...(product.composition_ids || [])],
    dosage_form: product.dosage_form || '',
    strength: product.strength || '',
    pack_size: product.pack_size || '',
    unit: product.unit || 'PCS',
    hsn_code: product.hsn_code || '',
    gst_rate: Number(product.gst_rate || 0),
    barcode: product.barcode || '',
    is_prescription_required: product.is_prescription_required,
    is_active: product.is_active,
    schedule_class: product.schedule_class || 'OTC',
    is_narcotic: product.is_narcotic || false,
    units_per_pack: Number(product.units_per_pack || 1),
  })
  unitsPerPack.value = Number(product.units_per_pack || 1)
  brandSearch.value = ''
  compositionSearch.value = ''
  newBrandName.value = ''
  newCompositionName.value = ''
  loadMasters()

  activeMenu.value = null
  clearMessages()
  dialog.value = 'edit'
}

function openView(product: Product) {
  selectedProduct.value = product
  activeMenu.value = null
  dialog.value = 'view'
}

function openDelete(product: Product) {
  selectedProduct.value = product
  activeMenu.value = null
  clearMessages()
  dialog.value = 'delete'
}

async function saveProduct() {
  if (!form.sku.trim() || !form.name.trim()) {
    errorMessage.value = 'SKU aur product name required hain.'
    return
  }

  const units = Number(form.units_per_pack || unitsPerPack.value || 1)
  if (!Number.isInteger(units) || units < 1) {
    errorMessage.value = 'Units per pack kam se kam 1 ka whole number hona chahiye.'
    return
  }

  const gst = Number(form.gst_rate ?? 0)

  if (!Number.isFinite(gst) || gst < 0 || gst > 100) {
    errorMessage.value = 'GST rate 0 se 100 ke beech hona chahiye.'
    return
  }

  saving.value = true
  errorMessage.value = ''

  const payload: ProductInput = {
    sku: form.sku.trim(),
    name: form.name.trim(),
    generic_name: null,
    brand_name: form.brand_name?.trim() || (brands.value.find(item => item.id === form.brand_id)?.name ?? null),
    brand_id: form.brand_id || null,
    composition_ids: [...(form.composition_ids || [])],
    dosage_form: form.dosage_form?.trim() || null,
    strength: form.strength?.trim() || null,
    pack_size: form.pack_size?.trim() || null,
    unit: form.unit?.trim() || 'PCS',
    hsn_code: form.hsn_code?.trim() || null,
    gst_rate: gst,
    barcode: form.barcode?.trim() || null,
    is_prescription_required: Boolean(form.is_prescription_required),
    is_active: Boolean(form.is_active),
    schedule_class: (form.schedule_class || 'OTC') as MedicineSchedule,
    is_narcotic: Boolean(form.is_narcotic),
    units_per_pack: units,
  }

  try {
    if (dialog.value === 'edit' && selectedProduct.value) {
      await api.updateProduct(selectedProduct.value.id, payload)
      dialog.value = null
      showSuccess('Product successfully updated.')
    } else {
      await api.createProduct(payload)
      dialog.value = null
      page.value = 1
      showSuccess('Product successfully added.')
    }

    await loadProducts()
  } catch (error: unknown) {
    const apiError = error as {
      data?: { error?: string; message?: string }
      message?: string
    }

    errorMessage.value =
      apiError.data?.error ||
      apiError.data?.message ||
      apiError.message ||
      'Product save nahi hua. Backend logs check karo.'
  } finally {
    saving.value = false
  }
}

async function deleteProduct() {
  if (!selectedProduct.value) return

  saving.value = true
  errorMessage.value = ''

  try {
    await api.deleteProduct(selectedProduct.value.id)

    dialog.value = null
    showSuccess('Product deleted successfully.')

    if (products.value.length === 1 && page.value > 1) {
      page.value--
    }

    await loadProducts()
  } catch (error: unknown) {
    const apiError = error as {
      data?: { error?: string; message?: string }
      message?: string
    }

    errorMessage.value =
      apiError.data?.error ||
      apiError.data?.message ||
      apiError.message ||
      'Product delete nahi hua. Ho sakta hai product kisi transaction mein use ho raha ho.'
  } finally {
    saving.value = false
  }
}

async function copyText(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    activeMenu.value = null
    showSuccess(`Copied: ${value}`)
  } catch {
    errorMessage.value = 'Clipboard access nahi mil paaya.'
  }
}

function toggleMenu(id: string) {
  activeMenu.value = activeMenu.value === id ? null : id
}

function closeMenuOnOutsideClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('[data-product-menu]')) {
    activeMenu.value = null
  }
}

function formatNumber(value: number | string | null | undefined) {
  const number = Number(value ?? 0)
  return new Intl.NumberFormat('en-IN', {
    maximumFractionDigits: 2,
  }).format(Number.isFinite(number) ? number : 0)
}

function formatCurrency(value: number | string | null | undefined) {
  const number = Number(value ?? 0)
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2,
  }).format(Number.isFinite(number) ? number : 0)
}

function titleCase(value: string | null | undefined) {
  if (!value) return '—'
  return value
    .replace(/[_-]/g, ' ')
    .replace(/\b\w/g, letter => letter.toUpperCase())
}

watch([statusFilter, formFilter, prescriptionFilter], () => {
  // These filters apply to the products currently loaded from the API.
})

onMounted(() => {
  loadProducts()
  document.addEventListener('click', closeMenuOnOutsideClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', closeMenuOnOutsideClick)
  if (searchTimer) clearTimeout(searchTimer)
  if (messageTimer) clearTimeout(messageTimer)
})
</script>

<template>
  <main class="products-page">
    <!-- Page heading -->
    <header class="page-heading">
      <div class="heading-copy">
        <div class="eyebrow">
          <Package :size="14" />
          <span>CATALOGUE MANAGEMENT</span>
        </div>

        <h1>Products</h1>
        <p>Manage your medicine catalogue, product details and GST information.</p>
      </div>

      <div class="heading-actions">
        <button class="button button-secondary" :disabled="loading" @click="loadProducts">
          <RefreshCw :size="15" :class="{ spinning: loading }" />
          Refresh
        </button>

        <button class="button button-primary" @click="openCreate">
          <Plus :size="16" />
          Add product
        </button>
      </div>
    </header>

    <!-- Compact stats -->
    <section class="stats-row">
      <article class="stat-card">
        <span class="stat-icon">
          <Package :size="17" />
        </span>
        <div class="stat-content">
          <span class="stat-label">Total products</span>
          <strong>{{ formatNumber(total) }}</strong>
        </div>
      </article>

      <article class="stat-card">
        <span class="stat-icon stat-icon-green">
          <Check :size="17" />
        </span>
        <div class="stat-content">
          <span class="stat-label">Active on page</span>
          <strong>{{ formatNumber(activeOnPage) }}</strong>
        </div>
      </article>

      <article class="stat-card">
        <span class="stat-icon stat-icon-muted">
          <AlertCircle :size="17" />
        </span>
        <div class="stat-content">
          <span class="stat-label">Inactive on page</span>
          <strong>{{ formatNumber(inactiveOnPage) }}</strong>
        </div>
      </article>

      <!-- <div class="stats-note">
        <span class="live-dot" />
        Connected to product database
      </div> -->
    </section>

    <!-- Notifications -->
    <div v-if="successMessage" class="notice notice-success">
      <Check :size="16" />
      <span>{{ successMessage }}</span>
      <button aria-label="Dismiss message" @click="successMessage = ''">
        <X :size="15" />
      </button>
    </div>

    <div v-if="errorMessage && !dialog" class="notice notice-error">
      <AlertCircle :size="16" />
      <span>{{ errorMessage }}</span>
      <button aria-label="Dismiss message" @click="errorMessage = ''">
        <X :size="15" />
      </button>
    </div>

    <!-- Product table -->
    <section class="table-panel">
      <div class="table-panel-heading">
        <div>
          <h2>Product register</h2>
          <p>Search, review and manage individual products.</p>
        </div>

        <span class="record-count">
          {{ formatNumber(total) }} records
        </span>
      </div>

      <!-- Toolbar -->
      <div class="toolbar">
        <label class="search-box">
          <Search :size="17" />
          <input
            v-model="search"
            type="search"
            placeholder="Search product, SKU, brand, composition or barcode..."
            @input="onSearchInput"
          />
          <kbd>Search</kbd>
        </label>

        <label class="filter-control">
          <Filter :size="15" />
          <select v-model="statusFilter" aria-label="Filter product status">
            <option value="all">All statuses</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </label>

        <label class="filter-control form-filter">
          <FlaskConical :size="15" />
          <select v-model="formFilter" aria-label="Filter dosage form">
            <option value="all">All forms</option>
            <option v-for="formName in dosageForms" :key="formName" :value="formName">
              {{ titleCase(formName) }}
            </option>
          </select>
        </label>

        <label class="filter-control prescription-filter">
          <select v-model="prescriptionFilter" aria-label="Filter prescription requirement">
            <option value="all">All prescriptions</option>
            <option value="required">Prescription required</option>
            <option value="not_required">No prescription</option>
          </select>
        </label>

        <button class="reset-button" @click="resetFilters">
          <RefreshCw :size="14" />
          Reset
        </button>
      </div>

      <!-- Loading -->
      <div v-if="loading && products.length === 0" class="loading-state">
        <LoaderCircle :size="25" class="spinning" />
        <span>Loading products...</span>
      </div>

      <!-- Error -->
      <div v-else-if="errorMessage && products.length === 0" class="empty-state">
        <span class="empty-icon empty-icon-error">
          <AlertCircle :size="23" />
        </span>
        <h3>Products couldn't be loaded</h3>
        <p>{{ errorMessage }}</p>
        <button class="button button-secondary" @click="loadProducts">
          <RefreshCw :size="15" />
          Try again
        </button>
      </div>

      <!-- Empty -->
      <div v-else-if="filteredProducts.length === 0" class="empty-state">
        <span class="empty-icon">
          <Package :size="24" />
        </span>
        <h3>{{ total === 0 ? 'No products found' : 'No matching products' }}</h3>
        <p>
          {{ total === 0
            ? 'The API returned no products. Check your database connection and search query.'
            : 'Try changing the search term or filters.' }}
        </p>
        <button
          v-if="search || statusFilter !== 'all' || formFilter !== 'all' || prescriptionFilter !== 'all'"
          class="button button-secondary"
          @click="resetFilters"
        >
          Clear filters
        </button>
        <button v-else class="button button-primary" @click="openCreate">
          <Plus :size="15" />
          Add first product
        </button>
      </div>

      <!-- Table -->
      <div v-else class="table-scroll">
        <table class="products-table">
          <thead>
            <tr>
              <th class="col-product">PRODUCT</th>
              <th class="col-sku">SKU / BARCODE</th>
              <th class="col-brand">BRAND / GENERIC</th>
              <th class="col-form">FORM / STRENGTH</th>
              <th class="col-pack">PACK</th>
              <th class="col-gst">GST</th>
              <th class="col-status">STATUS</th>
              <th class="col-actions"> </th>
            </tr>
          </thead>

          <tbody>
            <tr v-for="product in filteredProducts" :key="product.id">
              <td class="col-product">
                <div class="product-cell">
                  <span class="product-avatar">
                    <Package :size="17" />
                  </span>
                  <div class="product-copy">
                    <button class="product-name" @click="openView(product)">
                      {{ product.name }}
                    </button>
                    <span class="product-subtitle">
                      {{ product.is_prescription_required ? 'Prescription required' : 'Non-prescription' }}
                    </span>
                  </div>
                </div>
              </td>

              <td class="col-sku">
                <span class="primary-text">{{ product.sku }}</span>
                <span class="secondary-text barcode-text">
                  <Barcode v-if="product.barcode" :size="12" />
                  {{ product.barcode || 'No barcode' }}
                </span>
              </td>

              <td class="col-brand">
                <span class="primary-text">{{ product.brand_name || '—' }}</span>
                <span class="secondary-text">{{ (product.composition_names || []).join(' + ') || 'No composition linked' }}</span>
              </td>

              <td class="col-form">
                <span class="primary-text">{{ titleCase(product.dosage_form) }}</span>
                <span class="secondary-text">{{ product.strength || '—' }}</span>
              </td>

              <td class="col-pack">
                <span class="primary-text">{{ product.pack_size || '—' }}</span>
                <span class="secondary-text">{{ product.unit || 'PCS' }}</span>
              </td>

              <td class="col-gst">
                <span class="gst-value">{{ formatNumber(product.gst_rate) }}%</span>
                <span class="secondary-text">{{ product.hsn_code || 'No HSN' }}</span>
              </td>

              <td class="col-status">
                <span
                  class="status-pill"
                  :class="product.is_active ? 'status-active' : 'status-inactive'"
                >
                  <span class="status-dot" />
                  {{ product.is_active ? 'Active' : 'Inactive' }}
                </span>
              </td>

              <td class="col-actions">
                <div class="menu-wrapper" data-product-menu>
                  <button
                    class="icon-button menu-trigger"
                    :aria-label="`Actions for ${product.name}`"
                    :aria-expanded="activeMenu === product.id"
                    @click.stop="toggleMenu(product.id)"
                  >
                    <MoreHorizontal :size="19" />
                  </button>

                  <Transition name="menu">
                    <div v-if="activeMenu === product.id" class="action-menu">
                      <div class="action-menu-heading">Product actions</div>

                      <button @click="openView(product)">
                        <Eye :size="15" />
                        View product details
                      </button>

                      <button @click="openEdit(product)">
                        <Pencil :size="15" />
                        Edit product
                      </button>

                      <button @click="copyText(product.sku)">
                        <Copy :size="15" />
                        Copy SKU
                      </button>

                      <button
                        v-if="product.barcode"
                        @click="copyText(product.barcode)"
                      >
                        <Barcode :size="15" />
                        Copy barcode
                      </button>

                      <div class="action-menu-divider" />

                      <button class="action-danger" @click="openDelete(product)">
                        <Trash2 :size="15" />
                        Delete product
                      </button>
                    </div>
                  </Transition>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Footer / pagination -->
      <footer class="table-footer">
        <div class="pagination-summary">
          <span v-if="total > 0">
            Showing <strong>{{ formatNumber(startRecord) }}–{{ formatNumber(endRecord) }}</strong>
            of <strong>{{ formatNumber(total) }}</strong> products
          </span>
          <span v-else>Showing 0 products</span>
          <span v-if="loading" class="refreshing-label">
            <LoaderCircle :size="12" class="spinning" />
            Updating
          </span>
        </div>

        <div class="pagination-controls">
          <label class="rows-select">
            Rows
            <select :value="limit" @change="changeLimit">
              <option :value="25">25</option>
              <option :value="50">50</option>
              <option :value="100">100</option>
            </select>
          </label>

          <button
            class="pagination-button"
            :disabled="page <= 1 || loading"
            aria-label="Previous page"
            @click="changePage(page - 1)"
          >
            <ChevronLeft :size="16" />
          </button>

          <span class="page-number">{{ page }} / {{ totalPages }}</span>

          <button
            class="pagination-button"
            :disabled="page >= totalPages || loading"
            aria-label="Next page"
            @click="changePage(page + 1)"
          >
            <ChevronRight :size="16" />
          </button>
        </div>
      </footer>
    </section>
<!-- Dedicated Product Add/Edit Modal -->
<ProductFormModal
  :model-value="dialog === 'create' || dialog === 'edit'"
  :mode="dialog === 'edit' ? 'edit' : 'create'"
  :form="form"
  :brands="brands"
  :compositions="compositions"
  :brand-search="brandSearch"
  :composition-search="compositionSearch"
  :new-brand-name="newBrandName"
  :new-composition-name="newCompositionName"
  :adding-brand="addingBrand"
  :adding-composition="addingComposition"
  :master-loading="masterLoading"
  :saving="saving"
  :error-message="errorMessage"
  @close="dialog = null"
  @save="saveProduct"
  @add-brand="addBrandInline"
  @add-composition="addCompositionInline"
  @update:brand-search="brandSearch = $event; loadMasters()"
  @update:composition-search="compositionSearch = $event; loadMasters()"
  @update:new-brand-name="newBrandName = $event"
  @update:new-composition-name="newCompositionName = $event"
/>

    <!-- Product details dialog -->
    <Teleport to="body">
      <div
        v-if="dialog === 'view' && selectedProduct"
        class="modal-backdrop"
        @click.self="dialog = null"
      >
        <section class="modal-card details-modal" role="dialog" aria-modal="true">
          <header class="modal-header">
            <div>
              <span class="modal-eyebrow">PRODUCT DETAILS</span>
              <h2>{{ selectedProduct.name }}</h2>
              <p>{{ selectedProduct.sku }}</p>
            </div>
            <button class="icon-button" aria-label="Close dialog" @click="dialog = null">
              <X :size="19" />
            </button>
          </header>

          <div class="details-status">
            <span
              class="status-pill"
              :class="selectedProduct.is_active ? 'status-active' : 'status-inactive'"
            >
              <span class="status-dot" />
              {{ selectedProduct.is_active ? 'Active' : 'Inactive' }}
            </span>
            <span class="details-prescription">
              {{ selectedProduct.is_prescription_required ? 'Prescription required' : 'Non-prescription' }}
            </span>
          </div>

          <div class="details-grid">
            <div class="detail-item">
              <span>Brand</span>
              <strong>{{ selectedProduct.brand_name || '—' }}</strong>
            </div>
            <div class="detail-item">
              <span>Composition</span>
              <strong>{{ (selectedProduct.composition_names || []).join(' + ') || '—' }}</strong>
            </div>
            <div class="detail-item">
              <span>Dosage form</span>
              <strong>{{ titleCase(selectedProduct.dosage_form) }}</strong>
            </div>
            <div class="detail-item">
              <span>Strength</span>
              <strong>{{ selectedProduct.strength || '—' }}</strong>
            </div>
            <div class="detail-item">
              <span>Pack size</span>
              <strong>{{ selectedProduct.pack_size || '—' }}</strong>
            </div>
            <div class="detail-item">
              <span>Unit</span>
              <strong>{{ selectedProduct.unit || 'PCS' }}</strong>
            </div>
            <div class="detail-item">
              <span>Units per pack</span>
              <strong>{{ formatNumber(selectedProduct.units_per_pack || 1) }}</strong>
            </div>
            <div class="detail-item">
              <span>Schedule class</span>
              <strong>{{ selectedProduct.schedule_class || 'OTC' }}</strong>
            </div>
            <div class="detail-item">
              <span>Narcotic classification</span>
              <strong>{{ selectedProduct.is_narcotic ? 'Yes' : 'No' }}</strong>
            </div>
            <div class="detail-item">
              <span>Barcode</span>
              <strong>{{ selectedProduct.barcode || '—' }}</strong>
            </div>
            <div class="detail-item">
              <span>HSN code</span>
              <strong>{{ selectedProduct.hsn_code || '—' }}</strong>
            </div>
            <div class="detail-item">
              <span>GST rate</span>
              <strong>{{ formatNumber(selectedProduct.gst_rate) }}%</strong>
            </div>
            <div class="detail-item">
              <span>Product ID</span>
              <strong class="id-value">{{ selectedProduct.id }}</strong>
            </div>
          </div>

          <footer class="modal-footer">
            <button class="button button-secondary" @click="copyText(selectedProduct!.sku)">
              <Copy :size="15" />
              Copy SKU
            </button>
            <button class="button button-primary" @click="openEdit(selectedProduct!)">
              <Pencil :size="15" />
              Edit product
            </button>
          </footer>
        </section>
      </div>
    </Teleport>

    <!-- Delete confirmation -->
    <Teleport to="body">
      <div
        v-if="dialog === 'delete' && selectedProduct"
        class="modal-backdrop"
        @click.self="dialog = null"
      >
        <section class="modal-card confirm-modal" role="alertdialog" aria-modal="true">
          <span class="confirm-icon">
            <Trash2 :size="22" />
          </span>

          <h2>Delete product?</h2>
          <p>
            You're about to delete <strong>{{ selectedProduct.name }}</strong>
            ({{ selectedProduct.sku }}). This action cannot be undone.
          </p>

          <div class="confirm-warning">
            Products referenced by other records may not be deletable.
          </div>

          <div v-if="errorMessage" class="form-error">
            <AlertCircle :size="15" />
            {{ errorMessage }}
          </div>

          <footer class="modal-footer">
            <button class="button button-secondary" :disabled="saving" @click="dialog = null">
              Cancel
            </button>
            <button class="button button-danger" :disabled="saving" @click="deleteProduct">
              <LoaderCircle v-if="saving" :size="15" class="spinning" />
              <Trash2 v-else :size="15" />
              {{ saving ? 'Deleting...' : 'Delete product' }}
            </button>
          </footer>
        </section>
      </div>
    </Teleport>
  </main>
</template>

<style scoped>
.products-page {
  --line: #e5e7eb;
  --line-soft: #edf0f3;
  --text: #17191c;
  --muted: #737b87;
  --subtle: #9299a3;
  --surface: #fff;
  --surface-soft: #f8f9fb;
  --green: #07875c;
  --green-bg: #e9f8f0;
  --red: #dc3434;
  --red-bg: #fff0f0;
  color: var(--text);
  width: 100%;
  min-width: 0;
  padding: 18px 20px 24px;
  font-size: 13px;
}

.products-page * {
  box-sizing: border-box;
}

.page-heading {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  margin-bottom: 17px;
}

.heading-copy {
  min-width: 0;
}

.eyebrow {
  display: flex;
  align-items: center;
  gap: 7px;
  color: #626d7b;
  font-size: 10px;
  font-weight: 750;
  letter-spacing: 1.4px;
  margin-bottom: 6px;
}

.page-heading h1 {
  margin: 0;
  font-size: 27px;
  line-height: 1.2;
  letter-spacing: -1.05px;
  font-weight: 750;
}

.page-heading p,
.table-panel-heading p,
.modal-header p {
  color: var(--muted);
  margin: 5px 0 0;
  font-size: 12px;
}

.heading-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 34px;
  padding: 0 12px;
  border-radius: 7px;
  border: 1px solid var(--line);
  font-size: 12px;
  font-weight: 650;
  cursor: pointer;
  white-space: nowrap;
  transition: background .15s, border-color .15s;
}

.button:disabled {
  opacity: .6;
  cursor: not-allowed;
}

.button-primary {
  background: #17191c;
  border-color: #17191c;
  color: #fff;
}

.button-primary:hover:not(:disabled) {
  background: #30343a;
}

.button-secondary {
  background: #fff;
  color: #343a43;
}

.button-secondary:hover:not(:disabled) {
  background: #f7f8fa;
  border-color: #cbd0d7;
}

.button-danger {
  background: #d92d36;
  border-color: #d92d36;
  color: #fff;
}

.stats-row {
  display: flex;
  align-items: stretch;
  gap: 9px;
  margin-bottom: 15px;
}

.stat-card {
  min-width: 0;
  flex: 1 1 0;
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 63px;
  padding: 10px 12px;
  border: 1px solid var(--line);
  background: #fff;
  border-radius: 9px;
}

.stat-icon {
  width: 32px;
  height: 32px;
  border-radius: 7px;
  background: #f1f2f3;
  color: #444b54;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.stat-icon-green {
  color: var(--green);
  background: #e9f8f0;
}

.stat-icon-muted {
  color: #737b87;
  background: #f2f3f5;
}

.stat-content {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}

.stat-label {
  color: #717986;
  font-size: 11px;
  white-space: nowrap;
}

.stat-content strong {
  font-size: 19px;
  line-height: 1.15;
  font-weight: 750;
  letter-spacing: -.4px;
}

.stats-note {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 7px;
  color: var(--muted);
  font-size: 11px;
  white-space: nowrap;
  padding-left: 8px;
}

.live-dot,
.status-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  flex-shrink: 0;
  border-radius: 50%;
  background: #12a66a;
}

.notice {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 10px 12px;
  border-radius: 8px;
  margin-bottom: 12px;
  font-size: 12px;
}

.notice > span {
  flex: 1;
}

.notice > button {
  border: 0;
  background: transparent;
  display: grid;
  place-items: center;
  cursor: pointer;
}

.notice-success {
  color: #08744f;
  background: #edfaf3;
  border: 1px solid #c7efda;
}

.notice-error {
  color: #bd2828;
  background: #fff3f3;
  border: 1px solid #ffd8d8;
}

.table-panel {
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--surface);
  overflow: visible;
}

.table-panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 13px 15px;
  border-bottom: 1px solid var(--line-soft);
}

.table-panel-heading h2 {
  font-size: 14px;
  font-weight: 750;
  letter-spacing: -.2px;
  margin: 0;
}

.record-count {
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 5px 8px;
  color: #515966;
  background: #fff;
  font-size: 11px;
  white-space: nowrap;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--line);
  background: #fff;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  flex: 1 1 300px;
  min-width: 130px;
  padding: 0 10px;
  border: 1px solid #dfe3e8;
  border-radius: 7px;
  color: #858c97;
  background: #fff;
}

.search-box:focus-within {
  border-color: #aeb5bf;
  box-shadow: 0 0 0 2px #17191c0a;
}

.search-box input {
  width: 100%;
  min-width: 0;
  height: 100%;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text);
  font-size: 12px;
}

.search-box input::placeholder {
  color: #939aa5;
}

.search-box kbd {
  color: #737b87;
  font-size: 10px;
  background: #f6f7f9;
  border: 1px solid #e6e8ec;
  border-radius: 4px;
  padding: 2px 5px;
}

.filter-control {
  height: 36px;
  display: flex;
  align-items: center;
  gap: 7px;
  border: 1px solid #dfe3e8;
  border-radius: 7px;
  padding: 0 9px;
  color: #727a85;
  background: #fff;
  flex-shrink: 0;
}

.filter-control select {
  max-width: 145px;
  min-width: 0;
  height: 100%;
  border: 0;
  outline: 0;
  background: transparent;
  color: #343a43;
  font-size: 11px;
  cursor: pointer;
}

.form-filter select {
  max-width: 115px;
}

.prescription-filter select {
  max-width: 135px;
}

.reset-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 34px;
  border: 0;
  padding: 0 7px;
  color: #515966;
  background: transparent;
  font-size: 11px;
  cursor: pointer;
  white-space: nowrap;
}

.reset-button:hover {
  color: #111;
}

.table-scroll {
  width: 100%;
  overflow-x: auto;
}

.products-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  text-align: left;
  white-space: nowrap;
}

.products-table thead th {
  height: 33px;
  padding: 0 11px;
  background: #fafafa;
  color: #6e7682;
  border-bottom: 1px solid var(--line);
  font-size: 9px;
  font-weight: 750;
  letter-spacing: .55px;
}

.products-table tbody td {
  height: 61px;
  padding: 7px 11px;
  border-bottom: 1px solid #edf0f3;
  vertical-align: middle;
}

.products-table tbody tr:last-child td {
  border-bottom: 0;
}

.products-table tbody tr:hover {
  background: #fafbfc;
}

.col-product {
  min-width: 235px;
  width: 25%;
}

.col-sku {
  min-width: 120px;
}

.col-brand {
  min-width: 145px;
}

.col-form {
  min-width: 110px;
}

.col-pack {
  min-width: 78px;
}

.col-gst {
  min-width: 70px;
}

.col-status {
  min-width: 90px;
}

.col-actions {
  width: 48px;
  min-width: 48px;
  text-align: center !important;
}

.product-cell {
  display: flex;
  align-items: center;
  gap: 9px;
  min-width: 0;
}

.product-avatar {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  border: 1px solid #e7e9ec;
  border-radius: 8px;
  color: #555d67;
  background: #f6f7f8;
}

.product-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.product-name {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
  color: #202329;
  background: transparent;
  border: 0;
  padding: 0;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.product-name:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.product-subtitle,
.secondary-text {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #858d98;
  font-size: 10px;
  line-height: 1.3;
}

.primary-text {
  display: block;
  max-width: 190px;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #303640;
  font-size: 11px;
  font-weight: 550;
}

.secondary-text {
  margin-top: 4px;
}

.barcode-text {
  display: flex;
  align-items: center;
}

.gst-value {
  color: #303640;
  font-size: 11px;
  font-weight: 700;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-radius: 20px;
  padding: 4px 8px;
  font-size: 10px;
  font-weight: 650;
}

.status-active {
  color: #07875c;
  background: #e8f8ef;
  border: 1px solid #b5ebce;
}

.status-inactive {
  color: #707781;
  background: #f1f2f4;
  border: 1px solid #dfe2e6;
}

.status-inactive .status-dot {
  background: #9399a2;
}

.menu-wrapper {
  position: relative;
  display: inline-flex;
}

.icon-button {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border: 1px solid transparent;
  border-radius: 7px;
  background: transparent;
  color: #626a75;
  cursor: pointer;
}

.icon-button:hover {
  background: #f4f5f6;
  border-color: #e8eaed;
  color: #1e2228;
}

.menu-trigger {
  width: 29px;
  height: 29px;
}

.action-menu {
  position: absolute;
  z-index: 30;
  top: calc(100% + 4px);
  right: 0;
  width: 218px;
  padding: 5px;
  background: #fff;
  border: 1px solid #e0e3e8;
  border-radius: 10px;
  box-shadow: 0 12px 32px #15191f1a, 0 2px 6px #15191f0a;
}

.action-menu-heading {
  padding: 8px 9px 7px;
  color: #8a919c;
  font-size: 10px;
  font-weight: 750;
  letter-spacing: .55px;
  text-transform: uppercase;
}

.action-menu > button {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 9px;
  padding: 9px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #424954;
  text-align: left;
  font-size: 12px;
  cursor: pointer;
}

.action-menu > button:hover {
  background: #f5f6f8;
  color: #17191c;
}

.action-menu-divider {
  height: 1px;
  margin: 5px 2px;
  background: #eceef1;
}

.action-menu > button.action-danger {
  color: #dc3434;
}

.action-menu > button.action-danger:hover {
  background: #fff1f1;
}

.menu-enter-active,
.menu-leave-active {
  transition: opacity .12s ease, transform .12s ease;
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.loading-state,
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  text-align: center;
  padding: 50px 20px;
  color: var(--muted);
}

.loading-state {
  flex-direction: row;
  gap: 10px;
  min-height: 180px;
  font-size: 12px;
}

.empty-icon {
  display: grid;
  place-items: center;
  width: 48px;
  height: 48px;
  border: 1px solid #e7e9ec;
  border-radius: 12px;
  background: #f8f9fa;
  color: #747b85;
  margin-bottom: 12px;
}

.empty-icon-error {
  color: #cf3636;
  background: #fff3f3;
  border-color: #ffdddd;
}

.empty-state h3 {
  margin: 0;
  color: #24282e;
  font-size: 13px;
  font-weight: 750;
}

.empty-state p {
  max-width: 440px;
  margin: 7px 0 15px;
  color: var(--muted);
  font-size: 12px;
  line-height: 1.6;
}

.table-footer {
  min-height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 7px 12px;
  border-top: 1px solid var(--line);
  background: #fff;
  border-radius: 0 0 10px 10px;
}

.pagination-summary {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #777f8a;
  font-size: 11px;
}

.pagination-summary strong {
  color: #343a43;
  font-weight: 700;
}

.refreshing-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #777f8a;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.rows-select {
  display: flex;
  align-items: center;
  gap: 7px;
  color: #777f8a;
  font-size: 11px;
}

.rows-select select {
  height: 29px;
  padding: 0 7px;
  border: 1px solid var(--line);
  border-radius: 6px;
  color: #333943;
  background: #fff;
  outline: none;
  font-size: 11px;
}

.pagination-button {
  display: grid;
  place-items: center;
  width: 29px;
  height: 29px;
  border: 1px solid var(--line);
  border-radius: 6px;
  color: #4c535e;
  background: #fff;
  cursor: pointer;
}

.pagination-button:hover:not(:disabled) {
  background: #f4f5f7;
}

.pagination-button:disabled {
  opacity: .4;
  cursor: not-allowed;
}

.page-number {
  min-width: 44px;
  text-align: center;
  color: #515966;
  font-size: 11px;
}

/* Modal */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
  background: rgb(17 20 25 / 40%);
  backdrop-filter: blur(2px);
}

.modal-card {
  width: min(100%, 650px);
  max-height: min(90vh, 850px);
  overflow: auto;
  border: 1px solid #e5e7eb;
  border-radius: 13px;
  background: #fff;
  box-shadow: 0 24px 80px #0002;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 15px;
  padding: 18px 20px 15px;
  border-bottom: 1px solid var(--line);
}

.modal-eyebrow {
  color: #7b838e;
  font-size: 9px;
  font-weight: 750;
  letter-spacing: 1.2px;
}

.modal-header h2 {
  margin: 5px 0 0;
  font-size: 21px;
  line-height: 1.3;
  letter-spacing: -.5px;
  overflow-wrap: anywhere;
}

.product-form {
  padding: 17px 20px 0;
}

.form-section-title {
  margin: 0 0 12px;
  color: #414852;
  font-size: 11px;
  font-weight: 750;
}

.form-section-title:not(:first-child) {
  padding-top: 18px;
  margin-top: 18px;
  border-top: 1px solid var(--line-soft);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 13px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.field > span {
  color: #555d68;
  font-size: 11px;
  font-weight: 650;
}

.field > span b {
  color: #dc3434;
}

.field input,
.field select {
  width: 100%;
  height: 36px;
  padding: 0 10px;
  border: 1px solid #dfe3e8;
  border-radius: 7px;
  outline: none;
  background: #fff;
  color: #262b32;
  font: inherit;
  font-size: 12px;
}

.field input:focus,
.field select:focus {
  border-color: #8c949f;
  box-shadow: 0 0 0 3px #17191c0a;
}

.field input::placeholder {
  color: #a0a6af;
}

.field-full {
  grid-column: 1 / -1;
}

.form-options {
  display: grid;
  gap: 12px;
  margin-top: 20px;
  padding: 13px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: #fafbfc;
}

.checkbox-row {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  cursor: pointer;
}

.checkbox-row input {
  width: 15px;
  height: 15px;
  margin: 2px 0 0;
  accent-color: #17191c;
  flex-shrink: 0;
}

.checkbox-row span {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.checkbox-row strong {
  color: #343a43;
  font-size: 11px;
}

.checkbox-row small {
  color: #858d98;
  font-size: 10px;
  line-height: 1.5;
}

.form-error {
  display: flex;
  align-items: flex-start;
  gap: 7px;
  padding: 10px;
  margin-top: 13px;
  border: 1px solid #ffd7d7;
  border-radius: 7px;
  color: #c62f2f;
  background: #fff4f4;
  font-size: 11px;
  line-height: 1.5;
}

.form-error svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 20px;
  margin-top: 18px;
  border-top: 1px solid var(--line);
}

.product-form .modal-footer {
  margin-left: -20px;
  margin-right: -20px;
}

.details-modal {
  width: min(100%, 570px);
}

.details-status {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 15px 20px;
  border-bottom: 1px solid var(--line-soft);
}

.details-prescription {
  color: #6f7782;
  font-size: 11px;
}

.details-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  padding: 4px 20px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
  padding: 13px 8px 13px 0;
  border-bottom: 1px solid #f0f1f3;
}

.detail-item span {
  color: #858d98;
  font-size: 10px;
}

.detail-item strong {
  color: #333942;
  font-size: 12px;
  font-weight: 650;
  overflow-wrap: anywhere;
}

.id-value {
  font-size: 10px !important;
}

.confirm-modal {
  width: min(100%, 410px);
  padding-top: 22px;
}

.confirm-icon {
  display: grid;
  place-items: center;
  width: 43px;
  height: 43px;
  margin: 0 20px 12px;
  border-radius: 10px;
  color: #d92d36;
  background: #fff0f0;
}

.confirm-modal h2 {
  margin: 0 20px;
  font-size: 19px;
  letter-spacing: -.4px;
}

.confirm-modal > p {
  margin: 9px 20px 13px;
  color: #6e7682;
  font-size: 12px;
  line-height: 1.7;
  overflow-wrap: anywhere;
}

.confirm-modal > p strong {
  color: #303640;
}

.confirm-warning {
  margin: 0 20px;
  padding: 10px;
  border-radius: 7px;
  color: #737b87;
  background: #f7f8fa;
  font-size: 10px;
  line-height: 1.6;
}

.confirm-modal .form-error {
  margin-left: 20px;
  margin-right: 20px;
}

.confirm-modal .modal-footer {
  margin-top: 17px;
}

.spinning {
  animation: spin .8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@media (max-width: 1100px) {
  .products-page {
    padding: 15px;
  }

  .stats-row {
    flex-wrap: wrap;
  }

  .stat-card {
    flex: 1 1 calc(33.333% - 9px);
  }

  .stats-note {
    min-height: 28px;
  }

  .toolbar {
    flex-wrap: wrap;
  }

  .search-box {
    flex-basis: 100%;
  }

  .filter-control {
    flex: 0 1 auto;
  }
}

@media (max-width: 650px) {
  .products-page {
    padding: 12px 10px 20px;
  }

  .page-heading {
    align-items: flex-start;
    flex-direction: column;
    gap: 12px;
  }

  .page-heading h1 {
    font-size: 24px;
  }

  .heading-actions {
    width: 100%;
  }

  .heading-actions .button {
    flex: 1;
  }

  .stat-card {
    flex: 1 1 calc(50% - 8px);
    padding: 9px;
    gap: 8px;
  }

  .stat-content strong {
    font-size: 17px;
  }

  .stats-note {
    padding-left: 2px;
  }

  .table-panel-heading {
    padding: 12px 10px;
  }

  .toolbar {
    padding: 9px;
    gap: 7px;
  }

  .filter-control {
    flex: 1 1 calc(50% - 7px);
    min-width: 0;
  }

  .filter-control select,
  .form-filter select,
  .prescription-filter select {
    max-width: none;
    width: 100%;
  }

  .reset-button {
    flex: 1 1 100%;
    justify-content: flex-start;
  }

  .table-footer {
    align-items: flex-start;
    flex-direction: column;
    padding: 10px;
  }

  .pagination-controls {
    width: 100%;
    justify-content: flex-end;
  }

  .form-grid,
  .details-grid {
    grid-template-columns: 1fr;
  }

  .modal-backdrop {
    padding: 8px;
  }

  .modal-header {
    padding: 15px;
  }

  .product-form {
    padding: 15px 15px 0;
  }

  .product-form .modal-footer {
    margin-left: -15px;
    margin-right: -15px;
  }

  .modal-footer {
    padding: 12px 15px;
  }

  .detail-item {
    padding: 10px 0;
  }
}


/* Product master relationship controls */
.inline-add-row { display:flex; gap:8px; margin-top:8px; }
.inline-add-row input { flex:1; min-width:0; height:34px; border:1px solid #dfe3e8; border-radius:7px; padding:0 10px; font:inherit; font-size:12px; }
.inline-add-row .button { white-space:nowrap; }
.composition-picker { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:6px; max-height:150px; overflow:auto; border:1px solid #e5e7eb; border-radius:7px; padding:8px; background:#fff; }
.composition-option { display:flex; align-items:center; gap:8px; min-width:0; padding:5px; font-size:12px; color:#343a43; }
.composition-option span { overflow:hidden; text-overflow:ellipsis; }
.composition-option input { accent-color:#17191c; }
.selected-compositions, .field-hint { display:block; margin-top:6px; color:#737b87; font-size:11px; line-height:1.45; }
@media (max-width: 640px) { .inline-add-row { flex-direction:column; } .composition-picker { grid-template-columns:1fr; } }



/* Compact searchable master dropdowns */
.master-select { position: relative; width: 100%; }
.master-select-trigger {
  width: 100%; min-height: 40px; display: flex; align-items: center; justify-content: space-between;
  gap: 12px; padding: 0 12px; border: 1px solid var(--border, #d9dde3); border-radius: 8px;
  background: #fff; color: #20242a; text-align: left; font: inherit; cursor: pointer;
}
.master-select-trigger .muted { color: #858b95; }
.master-chevron { transform: rotate(90deg); flex-shrink: 0; }
.master-select-menu {
  position: absolute; z-index: 80; top: calc(100% + 5px); left: 0; right: 0;
  max-height: 260px; overflow-y: auto; padding: 6px; background: #fff;
  border: 1px solid #d9dde3; border-radius: 10px; box-shadow: 0 12px 32px rgb(0 0 0 / 14%);
}
.master-search {
  position: sticky; top: 0; z-index: 1; display: block; width: 100%; height: 36px;
  margin-bottom: 5px; padding: 0 10px; border: 1px solid #e0e3e8; border-radius: 7px;
  background: #fff; color: #20242a; outline: none;
}
.master-search:focus { border-color: #a7adb6; }
.master-option {
  width: 100%; min-height: 35px; display: flex; align-items: center; justify-content: space-between;
  gap: 10px; padding: 7px 9px; border: 0; border-radius: 6px; background: transparent;
  color: #292d33; text-align: left; font: inherit; cursor: pointer;
}
.master-option:hover, .master-option.selected { background: #f0f2f4; }
.master-empty { margin: 8px; color: #777e88; font-size: 12px; }
.compact-add-row { margin-top: 8px; }
.compact-add-row input { min-width: 0; }
.schedule-options { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; }
.schedule-option {
  display: flex; align-items: center; gap: 8px; min-height: 42px; padding: 8px 10px;
  border: 1px solid #d9dde3; border-radius: 8px; background: #fff; cursor: pointer;
  font-size: 13px; font-weight: 500;
}
.schedule-option.selected { border-color: #777e88; background: #f4f5f6; }
.schedule-option input { margin: 0; accent-color: #333; }
@media (max-width: 560px) {
  .schedule-options { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .compact-add-row { align-items: stretch; }
}
</style>
