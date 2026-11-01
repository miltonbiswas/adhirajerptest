<script setup lang="ts">
useHead({ title: 'Inventory' })
import {
  ref,
  computed,
  watch,
  onMounted,
  onBeforeUnmount,
} from 'vue'
import {
  Package,
  Layers3,
  Boxes,
  TriangleAlert,
  CircleAlert,
  CalendarClock,
  CalendarX,
  IndianRupee,
  RefreshCw,
  Search,
  SlidersHorizontal,
  CalendarDays,
  RotateCcw,
  MoreHorizontal,
  History,
  ArrowDownToLine,
  ArrowUpFromLine,
  PencilLine,
  Eye,
  X,
  ChevronLeft,
  ChevronRight,
  Check,
  Plus,
  Clock3,
} from 'lucide-vue-next'

const API = 'http://127.0.0.1:8080/api'

interface InventorySummary {
  total_products: number
  total_batches: number
  total_units: number
  low_stock: number
  out_of_stock: number
  expiring_soon: number
  expired: number
  stock_value: number
}

interface InventoryBatch {
  id: string
  product_id: string
  product_name: string
  sku: string
  brand_name: string | null
  batch_number: string
  manufacturing_date: string | null
  expiry_date: string | null
  quantity: number
  reorder_level: number
  unit: string
  purchase_rate: number
  mrp: number
  selling_rate: number
}

interface Product {
  id: string
  sku: string
  name: string
  generic_name?: string | null
  brand_name?: string | null
  dosage_form?: string | null
  strength?: string | null
  pack_size?: string | null
  unit?: string
  is_active?: boolean
}

interface PageResponse<T> {
  data: T[]
  page: number
  limit: number
  total: number
}

interface Movement {
  id: string
  product_id: string
  batch_id: string
  product_name: string
  sku: string
  batch_number: string
  movement_type: string
  quantity_change: number
  quantity_before: number
  quantity_after: number
  reason: string
  created_at: string
}

type StockFilter = 'all' | 'in_stock' | 'low_stock' | 'out_of_stock'
type ExpiryFilter = 'all' | 'valid' | 'expiring_30' | 'expiring_90' | 'expired'

const summary = ref<InventorySummary>({
  total_products: 0,
  total_batches: 0,
  total_units: 0,
  low_stock: 0,
  out_of_stock: 0,
  expiring_soon: 0,
  expired: 0,
  stock_value: 0,
})

const batches = ref<InventoryBatch[]>([])
const products = ref<Product[]>([])
const totalBatches = ref(0)
const totalProducts = ref(0)

const loading = ref(false)
const loadingSummary = ref(false)
const errorMessage = ref('')
const summaryError = ref('')
const lastUpdated = ref<Date | null>(null)

const search = ref('')
const stockFilter = ref<StockFilter>('all')
const expiryFilter = ref<ExpiryFilter>('all')
const page = ref(1)
const pageSize = ref(25)

const openDropdown = ref<'stock' | 'expiry' | 'rows' | null>(null)
const activeMenu = ref<string | null>(null)

const showAdjustment = ref(false)
const showHistory = ref(false)
const selectedBatch = ref<InventoryBatch | null>(null)
const selectedProduct = ref<Product | null>(null)
const movementHistory = ref<Movement[]>([])
const historyLoading = ref(false)
const actionLoading = ref(false)
const actionError = ref('')
const actionSuccess = ref('')

const adjustment = ref({
  movement_type: 'IN',
  quantity: 1,
  reason: '',
})

const stockOptions: { label: string; value: StockFilter }[] = [
  { label: 'All stock', value: 'all' },
  { label: 'In stock', value: 'in_stock' },
  { label: 'Low stock', value: 'low_stock' },
  { label: 'Out of stock', value: 'out_of_stock' },
]

const expiryOptions: { label: string; value: ExpiryFilter }[] = [
  { label: 'All expiry dates', value: 'all' },
  { label: 'Valid batches', value: 'valid' },
  { label: 'Expiring in 30 days', value: 'expiring_30' },
  { label: 'Expiring in 90 days', value: 'expiring_90' },
  { label: 'Expired batches', value: 'expired' },
]

const rowOptions = [10, 25, 50, 100]

const stockLabel = computed(
  () => stockOptions.find(x => x.value === stockFilter.value)?.label ?? 'All stock',
)

const expiryLabel = computed(
  () => expiryOptions.find(x => x.value === expiryFilter.value)?.label ?? 'All expiry dates',
)

const pageCount = computed(() =>
  Math.max(1, Math.ceil(totalBatches.value / pageSize.value)),
)

const startRecord = computed(() =>
  totalBatches.value === 0 ? 0 : (page.value - 1) * pageSize.value + 1,
)

const endRecord = computed(() =>
  Math.min(page.value * pageSize.value, totalBatches.value),
)

const catalogueStart = computed(() =>
  totalProducts.value === 0 ? 0 : (page.value - 1) * pageSize.value + 1,
)

const catalogueEnd = computed(() =>
  Math.min(page.value * pageSize.value, totalProducts.value),
)

const isCatalogueMode = computed(() => batches.value.length === 0 && products.value.length > 0)

const isCompletelyEmpty = computed(() =>
  !loading.value &&
  !search.value.trim() &&
  batches.value.length === 0 &&
  products.value.length === 0 &&
  !errorMessage.value,
)

function number(value: number | string | null | undefined): string {
  const n = Number(value ?? 0)
  if (!Number.isFinite(n)) return '0'
  return new Intl.NumberFormat('en-IN', {
    maximumFractionDigits: 3,
  }).format(n)
}

function currency(value: number | string | null | undefined): string {
  const n = Number(value ?? 0)
  if (!Number.isFinite(n)) return '₹0.00'
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(n)
}

function dateLabel(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en-IN', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  }).format(date)
}

function timeLabel(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en-IN', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

function expiryState(value: string | null | undefined): 'expired' | 'soon' | 'valid' | 'unknown' {
  if (!value) return 'unknown'

  const expiry = new Date(`${value.slice(0, 10)}T00:00:00`)
  if (Number.isNaN(expiry.getTime())) return 'unknown'

  const today = new Date()
  today.setHours(0, 0, 0, 0)

  const days = Math.ceil((expiry.getTime() - today.getTime()) / 86400000)

  if (days < 0) return 'expired'
  if (days <= 90) return 'soon'
  return 'valid'
}

function batchStockState(batch: InventoryBatch): 'out' | 'low' | 'in' {
  const quantity = Number(batch.quantity ?? 0)
  const reorder = Number(batch.reorder_level ?? 0)

  if (quantity <= 0) return 'out'
  if (reorder > 0 && quantity <= reorder) return 'low'
  return 'in'
}

function productDescription(product: Product): string {
  return [
    product.generic_name,
    product.strength,
    product.dosage_form,
    product.pack_size,
  ].filter(Boolean).join(' · ')
}

async function request<T>(path: string, options: Record<string, unknown> = {}): Promise<T> {
  return await $fetch<T>(`${API}${path}`, {
    ...options,
    timeout: 20000,
  } as Parameters<typeof $fetch<T>>[1])
}

async function loadSummary() {
  loadingSummary.value = true
  summaryError.value = ''

  try {
    summary.value = await request<InventorySummary>('/inventory/summary')
  } catch (error: unknown) {
    summaryError.value = getError(error)
  } finally {
    loadingSummary.value = false
  }
}

async function loadInventory() {
  loading.value = true
  errorMessage.value = ''
  activeMenu.value = null

  try {
    const params = {
      search: search.value.trim() || undefined,
      page: page.value,
      limit: pageSize.value,
      stock_status: stockFilter.value,
      expiry_status: expiryFilter.value,
    }

    const result = await request<PageResponse<InventoryBatch>>('/inventory/batches', {
      query: params,
    })

    batches.value = result.data ?? []
    totalBatches.value = Number(result.total ?? 0)

    // When there are no batches, show the product catalogue as a separate view.
    // This helps users browse imported products before opening stock records.
    if (batches.value.length === 0) {
      await loadCatalogue()
    } else {
      products.value = []
      totalProducts.value = 0
    }

    lastUpdated.value = new Date()
  } catch (error: unknown) {
    errorMessage.value = getError(error)
    batches.value = []
    totalBatches.value = 0

    // Keep catalogue browsing usable even if the inventory endpoint fails.
    try {
      await loadCatalogue()
    } catch {
      products.value = []
      totalProducts.value = 0
    }
  } finally {
    loading.value = false
  }
}

async function loadCatalogue() {
  const result = await request<PageResponse<Product>>('/products', {
    query: {
      search: search.value.trim() || undefined,
      page: page.value,
      limit: pageSize.value,
    },
  })

  products.value = (result.data ?? []).filter(product => product.is_active !== false)
  totalProducts.value = Number(result.total ?? 0)
}

async function refreshAll() {
  await Promise.all([loadSummary(), loadInventory()])
}

function setStockFilter(value: StockFilter) {
  stockFilter.value = value
  page.value = 1
  openDropdown.value = null
  void loadInventory()
}

function setExpiryFilter(value: ExpiryFilter) {
  expiryFilter.value = value
  page.value = 1
  openDropdown.value = null
  void loadInventory()
}

function setPageSize(value: number) {
  pageSize.value = value
  page.value = 1
  openDropdown.value = null
  void loadInventory()
}

function resetFilters() {
  search.value = ''
  stockFilter.value = 'all'
  expiryFilter.value = 'all'
  page.value = 1
  openDropdown.value = null
  void loadInventory()
}

function goToPage(value: number) {
  if (value < 1 || value > pageCount.value) return
  page.value = value
  void loadInventory()
}

function toggleDropdown(name: 'stock' | 'expiry' | 'rows') {
  openDropdown.value = openDropdown.value === name ? null : name
  activeMenu.value = null
}

function toggleMenu(id: string) {
  activeMenu.value = activeMenu.value === id ? null : id
  openDropdown.value = null
}

function closePopups(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (!target?.closest('[data-popup-root]')) {
    openDropdown.value = null
    activeMenu.value = null
  }
}

function getError(error: unknown): string {
  if (error && typeof error === 'object' && 'data' in error) {
    const data = (error as { data?: { message?: string; error?: string } }).data
    if (data?.message) return data.message
    if (data?.error) return data.error
  }

  if (error instanceof Error) return error.message
  return 'Unable to connect to the inventory API.'
}

function openAdjustment(batch: InventoryBatch, type: 'IN' | 'OUT' | 'ADJUSTMENT') {
  selectedBatch.value = batch
  selectedProduct.value = null
  adjustment.value = {
    movement_type: type,
    quantity: type === 'ADJUSTMENT' ? Number(batch.quantity) : 1,
    reason: '',
  }
  actionError.value = ''
  actionSuccess.value = ''
  showAdjustment.value = true
  activeMenu.value = null
}

function openProductAdjustment(product: Product) {
  selectedProduct.value = product
  selectedBatch.value = null
  actionError.value = ''
  actionSuccess.value = ''
  showAdjustment.value = true
  activeMenu.value = null
}

async function submitAdjustment() {
  actionError.value = ''
  actionSuccess.value = ''

  if (!adjustment.value.reason.trim()) {
    actionError.value = 'Please enter a reason for this stock movement.'
    return
  }

  if (!Number.isFinite(Number(adjustment.value.quantity)) || Number(adjustment.value.quantity) < 0) {
    actionError.value = 'Enter a valid quantity.'
    return
  }

  if (adjustment.value.movement_type !== 'ADJUSTMENT' && Number(adjustment.value.quantity) <= 0) {
    actionError.value = 'Quantity must be greater than zero.'
    return
  }

  if (selectedBatch.value && adjustment.value.movement_type === 'OUT') {
    if (Number(adjustment.value.quantity) > Number(selectedBatch.value.quantity)) {
      actionError.value = 'Quantity cannot exceed the available stock.'
      return
    }
  }

  // Creating stock for an existing product requires a batch record first.
  // Do not fabricate a batch or expiry date.
  if (!selectedBatch.value && selectedProduct.value) {
    actionError.value = 'This product has no batch record. Add a batch through the purchase/batch workflow before adjusting stock.'
    return
  }

  if (!selectedBatch.value) {
    actionError.value = 'Select a batch before adjusting stock.'
    return
  }

  actionLoading.value = true

  try {
    await request('/inventory/adjustments', {
      method: 'POST',
      body: {
        product_id: selectedBatch.value.product_id,
        batch_id: selectedBatch.value.id,
        movement_type: adjustment.value.movement_type,
        quantity: Number(adjustment.value.quantity),
        reason: adjustment.value.reason.trim(),
      },
    })

    showAdjustment.value = false
    actionSuccess.value = 'Stock adjustment saved.'
    await refreshAll()
  } catch (error: unknown) {
    actionError.value = getError(error)
  } finally {
    actionLoading.value = false
  }
}

async function loadHistory(batch?: InventoryBatch) {
  selectedBatch.value = batch ?? null
  showHistory.value = true
  historyLoading.value = true
  movementHistory.value = []
  activeMenu.value = null

  try {
    const result = await request<PageResponse<Movement>>('/inventory/adjustments', {
      query: {
        page: 1,
        limit: 100,
        product_id: batch?.product_id,
        batch_id: batch?.id,
      },
    })

    movementHistory.value = result.data ?? []
  } catch (error: unknown) {
    actionError.value = getError(error)
    movementHistory.value = []
  } finally {
    historyLoading.value = false
  }
}

function cardClass(tone: string) {
  return `metric-icon metric-icon--${tone}`
}

let searchTimer: ReturnType<typeof setTimeout> | undefined

watch(search, () => {
  page.value = 1
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    void loadInventory()
  }, 300)
})

onMounted(() => {
  void refreshAll()
  document.addEventListener('click', closePopups)
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  document.removeEventListener('click', closePopups)
})
</script>

<template>
  <main class="inventory-page">
    <header class="page-header">
      <div class="page-heading">
        <div class="eyebrow">
          <Package :size="14" />
          <span>Inventory management</span>
        </div>
        <h1>Inventory</h1>
        <p>Stock levels, batches and expiry monitoring</p>
      </div>

      <button class="button button--secondary refresh-button" :disabled="loading || loadingSummary" @click="refreshAll">
        <RefreshCw :size="15" :class="{ spinning: loading || loadingSummary }" />
        Refresh
      </button>
    </header>

    <section class="overview-section">
      <div class="section-heading">
        <div>
          <h2>Stock overview</h2>
          <p v-if="lastUpdated">Updated {{ lastUpdated.toLocaleTimeString('en-IN', { hour: '2-digit', minute: '2-digit' }) }}</p>
          <p v-else>Live figures from your inventory database</p>
        </div>
        <span class="live-status" :class="{ 'live-status--error': summaryError }">
          <span class="live-dot" />
          {{ summaryError ? 'Summary unavailable' : 'Live database' }}
        </span>
      </div>

      <div v-if="summaryError" class="inline-error">
        {{ summaryError }}
      </div>

      <div class="metrics-grid">
        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('neutral')"><Package :size="16" /></span>
            <span class="metric-label">Active products</span>
          </div>
          <strong>{{ number(summary.total_products) }}</strong>
          <small>Product catalogue</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('neutral')"><Layers3 :size="16" /></span>
            <span class="metric-label">Active batches</span>
          </div>
          <strong>{{ number(summary.total_batches) }}</strong>
          <small>Recorded batch records</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('neutral')"><Boxes :size="16" /></span>
            <span class="metric-label">Available units</span>
          </div>
          <strong>{{ number(summary.total_units) }}</strong>
          <small>Actual recorded stock</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('amber')"><TriangleAlert :size="16" /></span>
            <span class="metric-label">Low stock</span>
          </div>
          <strong>{{ number(summary.low_stock) }}</strong>
          <small>Batches at reorder level</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('red')"><CircleAlert :size="16" /></span>
            <span class="metric-label">Out of stock</span>
          </div>
          <strong>{{ number(summary.out_of_stock) }}</strong>
          <small>Products with no recorded units</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('amber')"><CalendarClock :size="16" /></span>
            <span class="metric-label">Expiring soon</span>
          </div>
          <strong>{{ number(summary.expiring_soon) }}</strong>
          <small>Within 90 days</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('red')"><CalendarX :size="16" /></span>
            <span class="metric-label">Expired batches</span>
          </div>
          <strong>{{ number(summary.expired) }}</strong>
          <small>Review before dispensing</small>
        </article>

        <article class="metric-card">
          <div class="metric-top">
            <span :class="cardClass('neutral')"><IndianRupee :size="16" /></span>
            <span class="metric-label">Stock value</span>
          </div>
          <strong class="metric-currency">{{ currency(summary.stock_value) }}</strong>
          <small>Based on purchase rate</small>
        </article>
      </div>
    </section>

    <section class="register-card">
      <div class="register-header">
        <div>
          <h2>Stock register</h2>
          <p>Search medicines, review batches and manage stock movements.</p>
        </div>

        <div class="register-actions">
          <span class="record-count">
            {{ number(isCatalogueMode ? totalProducts : totalBatches) }}
            {{ isCatalogueMode ? 'products' : 'batches' }}
          </span>
          <button class="button button--secondary history-button" @click="loadHistory()">
            <History :size="15" />
            Stock history
          </button>
        </div>
      </div>

      <div class="filters-bar">
        <label class="search-field">
          <Search :size="17" />
          <input
            v-model="search"
            type="search"
            placeholder="Search medicine, SKU, brand or batch..."
            aria-label="Search inventory"
          />
          <kbd v-if="search" class="clear-search" @click="search = ''">Clear</kbd>
        </label>

        <div class="filter-control" data-popup-root>
          <button
            class="filter-button"
            :class="{ 'filter-button--active': stockFilter !== 'all' }"
            @click.stop="toggleDropdown('stock')"
          >
            <SlidersHorizontal :size="15" />
            <span>{{ stockLabel }}</span>
            <span class="chevron">⌄</span>
          </button>

          <div v-if="openDropdown === 'stock'" class="dropdown-menu">
            <div class="dropdown-title">Stock status</div>
            <button
              v-for="option in stockOptions"
              :key="option.value"
              class="dropdown-option"
              @click="setStockFilter(option.value)"
            >
              {{ option.label }}
              <Check v-if="stockFilter === option.value" :size="15" />
            </button>
          </div>
        </div>

        <div class="filter-control" data-popup-root>
          <button
            class="filter-button"
            :class="{ 'filter-button--active': expiryFilter !== 'all' }"
            @click.stop="toggleDropdown('expiry')"
          >
            <CalendarDays :size="15" />
            <span>{{ expiryLabel }}</span>
            <span class="chevron">⌄</span>
          </button>

          <div v-if="openDropdown === 'expiry'" class="dropdown-menu dropdown-menu--right">
            <div class="dropdown-title">Expiry status</div>
            <button
              v-for="option in expiryOptions"
              :key="option.value"
              class="dropdown-option"
              @click="setExpiryFilter(option.value)"
            >
              {{ option.label }}
              <Check v-if="expiryFilter === option.value" :size="15" />
            </button>
          </div>
        </div>

        <button
          class="reset-button"
          :disabled="!search && stockFilter === 'all' && expiryFilter === 'all'"
          @click="resetFilters"
        >
          <RotateCcw :size="14" />
          Reset
        </button>
      </div>

      <div v-if="errorMessage && !products.length" class="error-panel">
        <CircleAlert :size="20" />
        <div>
          <strong>Inventory API issue</strong>
          <p>{{ errorMessage }}</p>
          <button class="text-button" @click="refreshAll">Try again</button>
        </div>
      </div>

      <div class="table-scroll">
        <table class="inventory-table">
          <thead>
            <tr>
              <th>Medicine</th>
              <th>{{ isCatalogueMode ? 'Product details' : 'Batch / expiry' }}</th>
              <th>{{ isCatalogueMode ? 'SKU' : 'Available stock' }}</th>
              <th>{{ isCatalogueMode ? 'Brand' : 'Purchase rate' }}</th>
              <th>{{ isCatalogueMode ? 'Unit' : 'MRP' }}</th>
              <th>Status</th>
              <th class="actions-heading">Actions</th>
            </tr>
          </thead>

          <tbody v-if="loading">
            <tr v-for="i in 6" :key="`loading-${i}`" class="skeleton-row">
              <td colspan="7"><span class="skeleton-line" /></td>
            </tr>
          </tbody>

          <tbody v-else-if="batches.length">
            <tr v-for="batch in batches" :key="batch.id">
              <td>
                <div class="medicine-name">{{ batch.product_name }}</div>
                <div class="subtext">{{ batch.sku }}<span v-if="batch.brand_name"> · {{ batch.brand_name }}</span></div>
              </td>

              <td>
                <div class="primary-cell">{{ batch.batch_number || '—' }}</div>
                <div class="subtext">Expiry: {{ dateLabel(batch.expiry_date) }}</div>
              </td>

              <td>
                <div class="quantity-cell">
                  <strong>{{ number(batch.quantity) }}</strong>
                  <span>{{ batch.unit || 'PCS' }}</span>
                </div>
                <div v-if="Number(batch.reorder_level) > 0" class="subtext">
                  Reorder at {{ number(batch.reorder_level) }}
                </div>
              </td>

              <td>{{ currency(batch.purchase_rate) }}</td>
              <td>{{ currency(batch.mrp) }}</td>

              <td>
                <span
                  v-if="expiryState(batch.expiry_date) === 'expired'"
                  class="status-badge status-badge--red"
                >Expired</span>
                <span
                  v-else-if="batchStockState(batch) === 'out'"
                  class="status-badge status-badge--red"
                >Out of stock</span>
                <span
                  v-else-if="expiryState(batch.expiry_date) === 'soon'"
                  class="status-badge status-badge--amber"
                >Expiring soon</span>
                <span
                  v-else-if="batchStockState(batch) === 'low'"
                  class="status-badge status-badge--amber"
                >Low stock</span>
                <span
                  v-else
                  class="status-badge status-badge--green"
                >In stock</span>
              </td>

              <td class="actions-cell" data-popup-root>
                <button
                  class="icon-button"
                  :aria-label="`Actions for ${batch.product_name}`"
                  @click.stop="toggleMenu(batch.id)"
                >
                  <MoreHorizontal :size="19" />
                </button>

                <div v-if="activeMenu === batch.id" class="action-menu">
                  <div class="action-menu-title">Batch actions</div>
                  <button @click="openAdjustment(batch, 'IN')">
                    <ArrowDownToLine :size="16" />
                    Stock received
                  </button>
                  <button @click="openAdjustment(batch, 'OUT')">
                    <ArrowUpFromLine :size="16" />
                    Stock issued
                  </button>
                  <button @click="openAdjustment(batch, 'ADJUSTMENT')">
                    <PencilLine :size="16" />
                    Adjust quantity
                  </button>
                  <button @click="loadHistory(batch)">
                    <History :size="16" />
                    View stock history
                  </button>
                </div>
              </td>
            </tr>
          </tbody>

          <tbody v-else-if="products.length">
            <tr v-for="product in products" :key="product.id">
              <td>
                <div class="medicine-name">{{ product.name }}</div>
                <div class="subtext">{{ product.sku }}</div>
              </td>

              <td>
                <div class="primary-cell">{{ productDescription(product) || 'Details not provided' }}</div>
                <div class="subtext">No batch record</div>
              </td>

              <td><span class="not-recorded">Not recorded</span></td>
              <td>{{ product.brand_name || '—' }}</td>
              <td>{{ product.unit || 'PCS' }}</td>

              <td>
                <span class="status-badge status-badge--neutral">No batch</span>
              </td>

              <td class="actions-cell" data-popup-root>
                <button
                  class="icon-button"
                  :aria-label="`Actions for ${product.name}`"
                  @click.stop="toggleMenu(product.id)"
                >
                  <MoreHorizontal :size="19" />
                </button>

                <div v-if="activeMenu === product.id" class="action-menu">
                  <div class="action-menu-title">Product actions</div>
                  <button @click="openProductAdjustment(product)">
                    <Plus :size="16" />
                    Add stock
                  </button>
                  <p class="action-menu-note">
                    A batch must be created through the purchase/batch workflow first.
                  </p>
                </div>
              </td>
            </tr>
          </tbody>

          <tbody v-else>
            <tr>
              <td colspan="7">
                <div class="empty-state">
                  <span class="empty-icon">
                    <Package :size="24" />
                  </span>
                  <strong>{{ isCompletelyEmpty ? 'No products found' : 'No matching inventory records' }}</strong>
                  <p v-if="errorMessage">{{ errorMessage }}</p>
                  <p v-else-if="stockFilter !== 'all' || expiryFilter !== 'all'">
                    Try changing your filters or resetting them.
                  </p>
                  <p v-else-if="search">
                    No product or batch matches “{{ search }}”.
                  </p>
                  <p v-else>
                    Your catalogue or batch records could not be loaded.
                  </p>
                  <button class="button button--secondary" @click="resetFilters">
                    <RotateCcw :size="14" />
                    Reset filters
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <footer class="table-footer">
        <div class="pagination-summary">
          <template v-if="isCatalogueMode">
            Showing <strong>{{ number(catalogueStart) }}–{{ number(catalogueEnd) }}</strong>
            of <strong>{{ number(totalProducts) }}</strong> products
            <span class="catalogue-note"> · Catalogue view, no batch stock recorded</span>
          </template>
          <template v-else>
            Showing <strong>{{ number(startRecord) }}–{{ number(endRecord) }}</strong>
            of <strong>{{ number(totalBatches) }}</strong> batches
          </template>
        </div>

        <div class="pagination-controls">
          <div class="rows-control" data-popup-root>
            <span>Rows</span>
            <button class="rows-button" @click.stop="toggleDropdown('rows')">
              {{ pageSize }}
              <span class="chevron">⌄</span>
            </button>

            <div v-if="openDropdown === 'rows'" class="dropdown-menu dropdown-menu--up">
              <button
                v-for="size in rowOptions"
                :key="size"
                class="dropdown-option"
                @click="setPageSize(size)"
              >
                {{ size }} rows
                <Check v-if="pageSize === size" :size="15" />
              </button>
            </div>
          </div>

          <button
            class="pagination-button"
            :disabled="page <= 1 || loading"
            aria-label="Previous page"
            @click="goToPage(page - 1)"
          >
            <ChevronLeft :size="17" />
          </button>

          <span class="page-indicator">{{ page }} / {{ pageCount }}</span>

          <button
            class="pagination-button"
            :disabled="page >= pageCount || loading"
            aria-label="Next page"
            @click="goToPage(page + 1)"
          >
            <ChevronRight :size="17" />
          </button>
        </div>
      </footer>
    </section>

    <!-- Stock adjustment modal -->
    <Teleport to="body">
      <div v-if="showAdjustment" class="modal-backdrop" @click.self="showAdjustment = false">
        <section class="modal-card" role="dialog" aria-modal="true" aria-labelledby="adjustment-title">
          <header class="modal-header">
            <div>
              <h2 id="adjustment-title">Stock adjustment</h2>
              <p>{{ selectedBatch?.product_name || selectedProduct?.name }}</p>
            </div>
            <button class="icon-button" aria-label="Close dialog" @click="showAdjustment = false">
              <X :size="18" />
            </button>
          </header>

          <div v-if="selectedBatch" class="modal-batch-info">
            <span>Batch <strong>{{ selectedBatch.batch_number }}</strong></span>
            <span>Available <strong>{{ number(selectedBatch.quantity) }} {{ selectedBatch.unit }}</strong></span>
          </div>

          <form class="adjustment-form" @submit.prevent="submitAdjustment">
            <label class="form-field">
              <span>Movement type</span>
              <select v-model="adjustment.movement_type">
                <option value="IN">Stock received (IN)</option>
                <option value="OUT">Stock issued (OUT)</option>
                <option value="ADJUSTMENT">Set actual quantity</option>
              </select>
            </label>

            <label class="form-field">
              <span>{{ adjustment.movement_type === 'ADJUSTMENT' ? 'Actual quantity' : 'Quantity' }}</span>
              <input v-model.number="adjustment.quantity" type="number" min="0" step="0.001" required />
            </label>

            <label class="form-field">
              <span>Reason / reference</span>
              <textarea
                v-model="adjustment.reason"
                rows="3"
                placeholder="e.g. Physical stock count, purchase invoice..."
                required
              />
            </label>

            <div v-if="actionError" class="form-error">{{ actionError }}</div>

            <footer class="modal-footer">
              <button type="button" class="button button--secondary" @click="showAdjustment = false">
                Cancel
              </button>
              <button type="submit" class="button button--primary" :disabled="actionLoading">
                <RefreshCw v-if="actionLoading" :size="15" class="spinning" />
                {{ actionLoading ? 'Saving...' : 'Save adjustment' }}
              </button>
            </footer>
          </form>
        </section>
      </div>
    </Teleport>

    <!-- Stock history modal -->
    <Teleport to="body">
      <div v-if="showHistory" class="modal-backdrop" @click.self="showHistory = false">
        <section class="modal-card modal-card--wide" role="dialog" aria-modal="true" aria-labelledby="history-title">
          <header class="modal-header">
            <div>
              <h2 id="history-title">Stock movement history</h2>
              <p>{{ selectedBatch ? `${selectedBatch.product_name} · Batch ${selectedBatch.batch_number}` : 'Recent recorded stock adjustments' }}</p>
            </div>
            <button class="icon-button" aria-label="Close history" @click="showHistory = false">
              <X :size="18" />
            </button>
          </header>

          <div v-if="historyLoading" class="history-message">
            <RefreshCw :size="18" class="spinning" />
            Loading stock history...
          </div>

          <div v-else-if="movementHistory.length" class="history-table-wrap">
            <table class="inventory-table history-table">
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Medicine / batch</th>
                  <th>Movement</th>
                  <th>Change</th>
                  <th>Before → after</th>
                  <th>Reason</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="movement in movementHistory" :key="movement.id">
                  <td>{{ timeLabel(movement.created_at) }}</td>
                  <td>
                    <div class="medicine-name">{{ movement.product_name }}</div>
                    <div class="subtext">{{ movement.batch_number }}</div>
                  </td>
                  <td>
                    <span class="status-badge status-badge--neutral">{{ movement.movement_type }}</span>
                  </td>
                  <td :class="Number(movement.quantity_change) < 0 ? 'negative-value' : 'positive-value'">
                    {{ Number(movement.quantity_change) > 0 ? '+' : '' }}{{ number(movement.quantity_change) }}
                  </td>
                  <td>{{ number(movement.quantity_before) }} → {{ number(movement.quantity_after) }}</td>
                  <td>{{ movement.reason }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div v-else class="empty-state history-empty">
            <span class="empty-icon"><Clock3 :size="23" /></span>
            <strong>No stock movements recorded</strong>
            <p>Stock movements will appear here after an adjustment is saved.</p>
          </div>
        </section>
      </div>
    </Teleport>
  </main>
</template>

<style scoped>
.inventory-page {
  --ink: #17191d;
  --muted: #737b88;
  --line: #e6e8ec;
  --surface: #fff;
  --soft: #f7f8fa;
  --green: #16845b;
  --amber: #a96008;
  --red: #c83a3a;
  color: var(--ink);
  width: 100%;
  min-width: 0;
  padding: 16px 18px 22px;
  font-size: 13px;
}

.page-header,
.page-heading,
.section-heading,
.metric-top,
.register-header,
.register-actions,
.filters-bar,
.filter-button,
.reset-button,
.table-footer,
.pagination-controls,
.rows-control,
.modal-header,
.modal-footer {
  display: flex;
  align-items: center;
}

.page-header {
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 17px;
}

.page-heading {
  display: block;
}

.eyebrow {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 5px;
  color: #666f7d;
  font-size: 10px;
  font-weight: 750;
  letter-spacing: 1.35px;
  text-transform: uppercase;
}

.page-heading h1 {
  margin: 0;
  font-size: clamp(24px, 2.1vw, 30px);
  line-height: 1.2;
  font-weight: 760;
  letter-spacing: -1px;
}

.page-heading p,
.section-heading p,
.register-header p {
  margin: 4px 0 0;
  color: var(--muted);
  font-size: 12px;
}

.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 34px;
  padding: 0 12px;
  border: 1px solid var(--line);
  border-radius: 7px;
  font-size: 12px;
  font-weight: 650;
  white-space: nowrap;
  cursor: pointer;
  transition: background .15s, border-color .15s;
}

.button:disabled {
  cursor: not-allowed;
  opacity: .55;
}

.button--secondary {
  background: #fff;
  color: #303640;
}

.button--secondary:hover:not(:disabled) {
  background: #f6f7f9;
  border-color: #cbd0d8;
}

.button--primary {
  color: #fff;
  background: #17191d;
  border-color: #17191d;
}

.button--primary:hover:not(:disabled) {
  background: #30343a;
}

.refresh-button {
  min-height: 34px;
}

.overview-section {
  margin-bottom: 18px;
}

.section-heading {
  justify-content: space-between;
  margin-bottom: 9px;
}

.section-heading h2,
.register-header h2 {
  margin: 0;
  font-size: 14px;
  font-weight: 750;
  letter-spacing: -.2px;
}

.section-heading p {
  font-size: 11px;
}

.live-status {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  color: #6f7784;
  font-size: 10px;
  white-space: nowrap;
}

.live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #21a366;
}

.live-status--error .live-dot {
  background: var(--red);
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(155px, 1fr));
  gap: 9px;
}

.metric-card {
  min-width: 0;
  min-height: 98px;
  padding: 11px 12px 10px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.metric-top {
  gap: 8px;
  min-width: 0;
}

.metric-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 27px;
  width: 27px;
  height: 27px;
  border-radius: 7px;
}

.metric-icon--neutral {
  color: #424850;
  background: #f0f1f3;
}

.metric-icon--amber {
  color: var(--amber);
  background: #fff3df;
}

.metric-icon--red {
  color: var(--red);
  background: #ffeded;
}

.metric-label {
  overflow: hidden;
  color: #656d79;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.metric-card > strong {
  display: block;
  margin: 7px 0 3px;
  font-size: clamp(20px, 1.6vw, 24px);
  font-weight: 760;
  line-height: 1.15;
  letter-spacing: -.65px;
  font-variant-numeric: tabular-nums;
}

.metric-card > strong.metric-currency {
  font-size: clamp(17px, 1.35vw, 21px);
}

.metric-card > small {
  color: #8a909a;
  font-size: 10px;
  line-height: 1.35;
}

.register-card {
  overflow: visible;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 11px;
}

.register-header {
  justify-content: space-between;
  gap: 16px;
  padding: 13px 15px;
  border-bottom: 1px solid var(--line);
}

.register-header h2 {
  font-size: 14px;
}

.register-header p {
  font-size: 11px;
}

.register-actions {
  gap: 8px;
}

.record-count {
  display: inline-flex;
  align-items: center;
  min-height: 30px;
  padding: 0 9px;
  border: 1px solid var(--line);
  border-radius: 6px;
  color: #545c68;
  font-size: 11px;
  white-space: nowrap;
}

.history-button {
  min-height: 32px;
}

.filters-bar {
  gap: 8px;
  padding: 10px 13px;
  background: #fff;
  border-bottom: 1px solid var(--line);
}

.search-field {
  display: flex;
  align-items: center;
  gap: 9px;
  flex: 1 1 260px;
  min-width: 120px;
  height: 36px;
  padding: 0 10px;
  color: #858c97;
  border: 1px solid #dfe2e7;
  border-radius: 7px;
  background: #fff;
}

.search-field:focus-within {
  border-color: #a4abb5;
  box-shadow: 0 0 0 2px rgb(20 24 30 / 4%);
}

.search-field input {
  width: 100%;
  min-width: 0;
  padding: 0;
  color: var(--ink);
  border: 0;
  outline: 0;
  background: transparent;
  font: inherit;
  font-size: 12px;
}

.search-field input::placeholder {
  color: #9299a4;
}

.search-field input::-webkit-search-cancel-button {
  display: none;
}

.clear-search {
  padding: 3px 5px;
  color: #6f7784;
  cursor: pointer;
  font-size: 10px;
}

.filter-control {
  position: relative;
  flex: 0 0 auto;
}

.filter-button {
  justify-content: space-between;
  gap: 8px;
  height: 36px;
  max-width: 215px;
  padding: 0 10px;
  color: #3d444e;
  border: 1px solid #dfe2e7;
  border-radius: 7px;
  background: #fff;
  cursor: pointer;
  font-size: 11px;
  white-space: nowrap;
}

.filter-button span:nth-child(2) {
  overflow: hidden;
  text-overflow: ellipsis;
}

.filter-button:hover,
.filter-button--active {
  background: #f7f8fa;
  border-color: #c7ccd4;
}

.chevron {
  color: #818894;
  font-size: 15px;
  line-height: 1;
}

.reset-button {
  justify-content: center;
  gap: 6px;
  height: 34px;
  padding: 0 8px;
  color: #5e6672;
  border: 0;
  background: transparent;
  cursor: pointer;
  font-size: 11px;
  white-space: nowrap;
}

.reset-button:hover:not(:disabled) {
  color: #17191d;
  background: #f5f6f8;
  border-radius: 6px;
}

.reset-button:disabled {
  opacity: .45;
  cursor: default;
}

.dropdown-menu,
.action-menu {
  position: absolute;
  z-index: 30;
  top: calc(100% + 6px);
  left: 0;
  width: 205px;
  padding: 5px;
  background: #fff;
  border: 1px solid #e1e4e9;
  border-radius: 10px;
  box-shadow: 0 12px 32px rgb(18 25 38 / 12%), 0 2px 6px rgb(18 25 38 / 4%);
}

.dropdown-menu--right {
  right: 0;
  left: auto;
}

.dropdown-menu--up {
  top: auto;
  bottom: calc(100% + 6px);
  width: 125px;
}

.dropdown-title,
.action-menu-title {
  padding: 8px 9px 7px;
  color: #737b88;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: .3px;
}

.dropdown-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  min-height: 33px;
  gap: 8px;
  padding: 0 9px;
  color: #343a43;
  border: 0;
  border-radius: 6px;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-size: 12px;
}

.dropdown-option:hover {
  background: #f3f5f7;
}

.dropdown-option svg {
  flex-shrink: 0;
  color: #17191d;
}

.table-scroll {
  width: 100%;
  overflow-x: auto;
}

.inventory-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  text-align: left;
  font-size: 12px;
}

.inventory-table thead th {
  height: 35px;
  padding: 0 12px;
  color: #707783;
  background: #fafbfc;
  border-bottom: 1px solid var(--line);
  font-size: 9px;
  font-weight: 750;
  letter-spacing: .45px;
  text-transform: uppercase;
  white-space: nowrap;
}

.inventory-table thead th:first-child,
.inventory-table tbody td:first-child {
  padding-left: 15px;
}

.inventory-table tbody td {
  position: relative;
  height: 57px;
  padding: 9px 12px;
  color: #434a55;
  border-bottom: 1px solid #edf0f3;
  vertical-align: middle;
}

.inventory-table tbody tr:last-child td {
  border-bottom: 0;
}

.inventory-table tbody tr:hover td {
  background: #fcfcfd;
}

.medicine-name {
  max-width: 300px;
  color: #20242b;
  font-size: 12px;
  font-weight: 650;
  line-height: 1.4;
}

.subtext {
  margin-top: 3px;
  color: #8a919c;
  font-size: 10px;
  line-height: 1.35;
}

.primary-cell {
  color: #3e454f;
  font-size: 11px;
  font-weight: 600;
}

.quantity-cell {
  display: flex;
  align-items: baseline;
  gap: 5px;
  white-space: nowrap;
}

.quantity-cell strong {
  color: #20242b;
  font-size: 13px;
  font-weight: 750;
  font-variant-numeric: tabular-nums;
}

.quantity-cell span {
  color: #89909a;
  font-size: 10px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 2px 8px;
  border: 1px solid transparent;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 600;
  white-space: nowrap;
}

.status-badge--green {
  color: #138456;
  border-color: #a9e8cd;
  background: #eafaf2;
}

.status-badge--amber {
  color: #996006;
  border-color: #ffdfa0;
  background: #fff7e5;
}

.status-badge--red {
  color: #c43232;
  border-color: #ffc7c7;
  background: #fff0f0;
}

.status-badge--neutral {
  color: #626b77;
  border-color: #e1e4e9;
  background: #f4f5f7;
}

.not-recorded {
  color: #858c97;
  font-size: 11px;
  font-style: italic;
}

.actions-heading {
  width: 58px;
  text-align: center;
}

.actions-cell {
  width: 58px;
  text-align: center;
}

.icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 31px;
  height: 31px;
  padding: 0;
  color: #4e5662;
  border: 1px solid transparent;
  border-radius: 7px;
  background: transparent;
  cursor: pointer;
}

.icon-button:hover {
  background: #f2f4f6;
  border-color: #e6e8ec;
}

.action-menu {
  top: 40px;
  right: 10px;
  left: auto;
  width: 220px;
  padding: 5px;
  text-align: left;
}

.action-menu > button {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  min-height: 36px;
  padding: 0 9px;
  color: #3e4550;
  border: 0;
  border-radius: 6px;
  background: #fff;
  text-align: left;
  cursor: pointer;
  font-size: 12px;
}

.action-menu > button:hover {
  background: #f4f5f7;
}

.action-menu > button svg {
  color: #707987;
}

.action-menu-note {
  margin: 5px 5px 7px;
  padding: 8px;
  color: #737b88;
  background: #f7f8fa;
  border-radius: 6px;
  font-size: 10px;
  line-height: 1.5;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 205px;
  padding: 28px 16px;
  text-align: center;
}

.empty-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 45px;
  height: 45px;
  margin-bottom: 12px;
  color: #707783;
  background: #f7f8fa;
  border: 1px solid #e7e9ed;
  border-radius: 12px;
}

.empty-state > strong {
  color: #252a32;
  font-size: 13px;
  font-weight: 700;
}

.empty-state > p {
  max-width: 480px;
  margin: 6px 0 13px;
  color: #818894;
  font-size: 11px;
  line-height: 1.5;
}

.empty-state .button {
  min-height: 31px;
}

.table-footer {
  justify-content: space-between;
  gap: 15px;
  min-height: 47px;
  padding: 7px 14px;
  border-top: 1px solid var(--line);
}

.pagination-summary {
  color: #737b87;
  font-size: 11px;
}

.pagination-summary strong {
  color: #3c434d;
  font-weight: 700;
}

.catalogue-note {
  color: #9a7b46;
  font-size: 10px;
}

.pagination-controls {
  gap: 8px;
}

.rows-control {
  position: relative;
  gap: 7px;
  color: #777f8a;
  font-size: 10px;
}

.rows-button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-width: 56px;
  height: 29px;
  padding: 0 8px;
  color: #3d444e;
  border: 1px solid #dfe2e7;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  font-size: 11px;
}

.pagination-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 29px;
  padding: 0;
  color: #454c57;
  border: 1px solid #e0e3e8;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
}

.pagination-button:disabled {
  color: #b6bbc3;
  background: #fafbfc;
  cursor: not-allowed;
}

.page-indicator {
  min-width: 36px;
  color: #6e7682;
  text-align: center;
  font-size: 11px;
  white-space: nowrap;
}

.inline-error,
.error-panel,
.form-error {
  padding: 10px 12px;
  color: #b52f2f;
  background: #fff3f3;
  border: 1px solid #ffd5d5;
  border-radius: 7px;
  font-size: 11px;
}

.inline-error {
  margin-bottom: 9px;
}

.error-panel {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin: 12px;
}

.error-panel strong {
  font-size: 12px;
}

.error-panel p {
  margin: 4px 0;
  line-height: 1.5;
}

.text-button {
  padding: 3px 0;
  color: #9f2929;
  border: 0;
  background: transparent;
  cursor: pointer;
  font-weight: 700;
  font-size: 11px;
}

.skeleton-row td {
  height: 43px !important;
}

.skeleton-line {
  display: block;
  width: 62%;
  height: 9px;
  background: linear-gradient(90deg, #f0f1f3 25%, #e8eaee 50%, #f0f1f3 75%);
  background-size: 200% 100%;
  border-radius: 6px;
  animation: shimmer 1.4s infinite;
}

.modal-backdrop {
  position: fixed;
  z-index: 1000;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
  background: rgb(17 22 29 / 42%);
  backdrop-filter: blur(2px);
}

.modal-card {
  width: min(100%, 460px);
  max-height: min(90vh, 780px);
  overflow: auto;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 13px;
  box-shadow: 0 24px 80px rgb(0 0 0 / 20%);
}

.modal-card--wide {
  width: min(100%, 1050px);
}

.modal-header {
  justify-content: space-between;
  gap: 12px;
  padding: 16px 18px;
  border-bottom: 1px solid var(--line);
}

.modal-header h2 {
  margin: 0;
  color: #20242b;
  font-size: 16px;
  font-weight: 750;
}

.modal-header p {
  margin: 5px 0 0;
  color: #777f8a;
  font-size: 11px;
}

.modal-batch-info {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 8px;
  padding: 12px 18px;
  color: #707783;
  background: #f8f9fb;
  border-bottom: 1px solid var(--line);
  font-size: 11px;
}

.modal-batch-info strong {
  margin-left: 4px;
  color: #2c3139;
}

.adjustment-form {
  display: grid;
  gap: 14px;
  padding: 17px 18px 18px;
}

.form-field {
  display: grid;
  gap: 6px;
}

.form-field > span {
  color: #3b424d;
  font-size: 11px;
  font-weight: 700;
}

.form-field input,
.form-field select,
.form-field textarea {
  width: 100%;
  min-height: 37px;
  padding: 8px 10px;
  color: #262b33;
  border: 1px solid #dfe3e8;
  border-radius: 7px;
  outline: none;
  background: #fff;
  font: inherit;
  font-size: 12px;
}

.form-field textarea {
  resize: vertical;
}

.form-field input:focus,
.form-field select:focus,
.form-field textarea:focus {
  border-color: #9ca3ad;
  box-shadow: 0 0 0 2px rgb(23 25 29 / 5%);
}

.modal-footer {
  justify-content: flex-end;
  gap: 8px;
  margin-top: 2px;
}

.history-message {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 150px;
  color: #737b87;
  font-size: 12px;
}

.history-table-wrap {
  max-height: 65vh;
  overflow: auto;
}

.history-table {
  min-width: 700px;
}

.history-empty {
  min-height: 220px;
}

.negative-value {
  color: var(--red) !important;
  font-weight: 700;
}

.positive-value {
  color: var(--green) !important;
  font-weight: 700;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes shimmer {
  to { background-position: -200% 0; }
}

@media (min-width: 1500px) {
  .metrics-grid {
    grid-template-columns: repeat(8, minmax(0, 1fr));
  }
}

@media (max-width: 1050px) {
  .inventory-page {
    padding: 13px;
  }

  .metrics-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .filters-bar {
    flex-wrap: wrap;
  }

  .search-field {
    flex-basis: 100%;
  }

  .filter-button {
    max-width: none;
  }
}

@media (max-width: 650px) {
  .inventory-page {
    padding: 12px 9px 18px;
  }

  .page-header {
    align-items: flex-start;
  }

  .page-heading h1 {
    font-size: 25px;
  }

  .refresh-button {
    padding: 0 9px;
  }

  .metrics-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 7px;
  }

  .metric-card {
    min-height: 91px;
    padding: 9px;
  }

  .metric-card > strong {
    font-size: 21px;
  }

  .metric-label {
    font-size: 10px;
  }

  .register-header {
    align-items: flex-start;
    padding: 12px 10px;
  }

  .register-actions {
    align-items: flex-end;
    flex-direction: column;
    gap: 5px;
  }

  .record-count {
    min-height: 25px;
  }

  .history-button {
    min-height: 28px;
    padding: 0 7px;
    font-size: 10px;
  }

  .filters-bar {
    gap: 7px;
    padding: 9px;
  }

  .filter-control {
    flex: 1 1 calc(50% - 8px);
  }

  .filter-button {
    width: 100%;
    justify-content: flex-start;
    gap: 6px;
    padding: 0 7px;
    font-size: 10px;
  }

  .filter-button span:nth-child(2) {
    flex: 1;
    text-align: left;
  }

  .reset-button {
    flex: 0 0 auto;
  }

  .table-footer {
    align-items: flex-start;
    flex-direction: column;
    gap: 9px;
    padding: 10px;
  }

  .pagination-controls {
    align-self: flex-end;
  }

  .catalogue-note {
    display: block;
    margin-top: 3px;
  }
}
</style>