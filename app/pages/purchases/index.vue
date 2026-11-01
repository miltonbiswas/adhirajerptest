
<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import {
  Search, Plus, RefreshCw, ChevronLeft, ChevronRight,
  Eye, Pencil, Printer, Trash2, CheckCircle2, X,
  FileText, MoreHorizontal, Package, CalendarDays,
  IndianRupee, LoaderCircle,
} from 'lucide-vue-next'

useSeoMeta({ title: 'Purchase Register | AdhirajPOS' })

const API = 'http://127.0.0.1:8080/api'

type Purchase = {
  id: string
  purchase_number: string
  supplier_id: string
  supplier_name: string
  supplier_invoice_number: string
  invoice_date: string
  due_date: string | null
  status: 'DRAFT' | 'POSTED' | 'CANCELLED'
  grand_total: number
  paid_amount: number
  created_at?: string
}

type PurchaseItem = {
  id: string
  line_number: number
  product_id: string | null
  product_name: string
  sku: string
  batch_number: string
  manufacturing_date: string | null
  expiry_date: string | null
  quantity: number
  free_quantity: number
  purchase_rate: number
  mrp: number
  selling_rate: number
  discount_percent: number
  discount_amount: number
  gst_rate: number
  taxable_amount: number
  cgst_amount: number
  sgst_amount: number
  igst_amount: number
  total_amount: number
}

type PurchaseDetail = {
  purchase: Purchase & {
    tax_mode: string
    tax_type: string
    subtotal: number
    discount_amount: number
    taxable_amount: number
    tax_amount: number
    other_charges: number
    round_off: number
    notes: string | null
  }
  items: PurchaseItem[]
}

type PurchaseListResponse = {
  data: Purchase[]
  page: number
  limit: number
  total: number
}

const purchases = ref<Purchase[]>([])
const search = ref('')
const status = ref('ALL')
const page = ref(1)
const limit = 25
const total = ref(0)

const loading = ref(true)
const refreshing = ref(false)
const error = ref('')
const actionError = ref('')

const selectedPurchase = ref<PurchaseDetail | null>(null)
const detailLoading = ref(false)
const detailOpen = ref(false)

const actionLoadingId = ref('')
const deleteTarget = ref<Purchase | null>(null)
const deleteLoading = ref(false)
const postTarget = ref<Purchase | null>(null)
const postLoading = ref(false)

const openMenuId = ref('')

let searchTimer: ReturnType<typeof setTimeout> | undefined
let requestId = 0

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / limit)),
)

const startRecord = computed(() =>
  total.value ? (page.value - 1) * limit + 1 : 0,
)

const endRecord = computed(() =>
  Math.min(page.value * limit, total.value),
)

const draftCount = computed(() =>
  purchases.value.filter(x => x.status === 'DRAFT').length,
)

const postedCount = computed(() =>
  purchases.value.filter(x => x.status === 'POSTED').length,
)

const visibleAmount = computed(() =>
  purchases.value.reduce((sum, item) =>
    sum + Number(item.grand_total || 0), 0),
)

function money(value: number | string | null | undefined) {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value || 0))
}

function date(value?: string | null) {
  if (!value) return '—'

  const parsed = new Date(`${value.slice(0, 10)}T00:00:00`)
  if (Number.isNaN(parsed.getTime())) return '—'

  return new Intl.DateTimeFormat('en-IN', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  }).format(parsed)
}

function statusLabel(value: string) {
  return value.charAt(0) + value.slice(1).toLowerCase()
}

function errorMessage(err: any, fallback: string) {
  return err?.data?.error ||
    err?.data?.message ||
    err?.message ||
    fallback
}

async function loadPurchases(showRefresh = false) {
  const currentRequest = ++requestId

  loading.value = true
  refreshing.value = showRefresh
  error.value = ''

  try {
    const response = await $fetch<PurchaseListResponse>(
      `${API}/purchases`,
      {
        query: {
          search: search.value.trim() || undefined,
          status: status.value,
          page: page.value,
          limit,
        },
      },
    )

    if (currentRequest !== requestId) return

    purchases.value = response.data ?? []
    total.value = Number(response.total ?? 0)
  } catch (err) {
    if (currentRequest !== requestId) return

    error.value = errorMessage(err, 'Unable to load purchases.')
    purchases.value = []
    total.value = 0
  } finally {
    if (currentRequest === requestId) {
      loading.value = false
      refreshing.value = false
    }
  }
}

function onSearch() {
  if (searchTimer) clearTimeout(searchTimer)

  searchTimer = setTimeout(() => {
    page.value = 1
    loadPurchases()
  }, 300)
}

watch(status, () => {
  page.value = 1
  loadPurchases()
})

watch(page, () => loadPurchases())

async function openDetails(item: Purchase) {
  detailOpen.value = true
  detailLoading.value = true
  selectedPurchase.value = null
  openMenuId.value = ''

  try {
    selectedPurchase.value = await $fetch<PurchaseDetail>(
      `${API}/purchases/${item.id}`,
    )
  } catch (err) {
    actionError.value = errorMessage(err, 'Unable to load purchase details.')
    detailOpen.value = false
  } finally {
    detailLoading.value = false
  }
}

function editPurchase(item: Purchase) {
  openMenuId.value = ''

  if (item.status !== 'DRAFT') {
    actionError.value = 'Only draft purchases can be edited.'
    return
  }

  navigateTo({
    path: '/purchases/new',
    query: { edit: item.id },
  })
}

async function postPurchase(item: Purchase) {
  if (item.status !== 'DRAFT') return

  postLoading.value = true
  actionLoadingId.value = item.id
  actionError.value = ''

  try {
    await $fetch(`${API}/purchases/${item.id}`, {
      method: 'POST',
    })

    postTarget.value = null
    await loadPurchases()
  } catch (err) {
    actionError.value = errorMessage(err, 'Unable to post purchase.')
  } finally {
    postLoading.value = false
    actionLoadingId.value = ''
  }
}

async function deletePurchase() {
  if (!deleteTarget.value) return

  const item = deleteTarget.value
  deleteLoading.value = true
  actionError.value = ''

  try {
    await $fetch(`${API}/purchases/${item.id}`, {
      method: 'DELETE',
    })

    deleteTarget.value = null
    await loadPurchases()
  } catch (err) {
    actionError.value = errorMessage(
      err,
      'Unable to delete purchase. Check whether the DELETE API is implemented.',
    )
  } finally {
    deleteLoading.value = false
  }
}

function printPurchase(detail: PurchaseDetail) {
  const p = detail.purchase
  const items = detail.items || []

  const rows = items.map((item, index) => `
    <tr>
      <td>${index + 1}</td>
      <td>
        <strong>${escapeHtml(item.product_name)}</strong>
        <small>${escapeHtml(item.sku || '')}</small>
      </td>
      <td>${escapeHtml(item.batch_number)}</td>
      <td>${Number(item.quantity)}</td>
      <td>${Number(item.free_quantity)}</td>
      <td>${money(item.purchase_rate)}</td>
      <td>${Number(item.discount_percent)}%</td>
      <td class="right">${money(item.total_amount)}</td>
    </tr>
  `).join('')

  const html = `
  <!DOCTYPE html>
  <html>
  <head>
    <meta charset="UTF-8">
    <title>${escapeHtml(p.purchase_number)}</title>
    <style>
      * { box-sizing: border-box; }
      body {
        margin: 0; padding: 30px; color: #171717;
        font-family: Arial, sans-serif; font-size: 12px;
      }
      h1 { margin: 0 0 5px; font-size: 23px; }
      h2 { margin: 0 0 15px; font-size: 15px; }
      .muted { color: #666; }
      .top {
        display:flex; justify-content:space-between;
        padding-bottom:18px; border-bottom:2px solid #222;
      }
      .meta {
        display:grid; grid-template-columns:1fr 1fr;
        gap:12px; margin:20px 0;
      }
      .meta div { line-height:1.7; }
      table { width:100%; border-collapse:collapse; margin-top:20px; }
      th,td { padding:10px 8px; border-bottom:1px solid #ddd; text-align:left; }
      th { background:#f3f3f3; font-size:10px; text-transform:uppercase; }
      td small { display:block; margin-top:4px; color:#777; }
      .right { text-align:right; }
      .totals { width:310px; margin:20px 0 0 auto; }
      .totals div {
        display:flex; justify-content:space-between;
        padding:7px 0; border-bottom:1px solid #eee;
      }
      .grand { font-size:16px; font-weight:bold; border-top:2px solid #222; }
      .notes { margin-top:25px; padding-top:12px; border-top:1px solid #ddd; }
      @media print {
        body { padding:0; }
        @page { size:A4; margin:12mm; }
      }
    </style>
  </head>
  <body>
    <div class="top">
      <div>
        <h1>Purchase Invoice</h1>
        <div class="muted">AdhirajPOS · Purchase Register</div>
      </div>
      <div style="text-align:right">
        <h2>${escapeHtml(p.purchase_number)}</h2>
        <div>${escapeHtml(p.status)}</div>
      </div>
    </div>

    <div class="meta">
      <div>
        <strong>Supplier</strong><br>
        ${escapeHtml(p.supplier_name)}<br>
        <span class="muted">Supplier Bill: ${escapeHtml(p.supplier_invoice_number)}</span>
      </div>
      <div>
        <strong>Invoice Details</strong><br>
        Invoice Date: ${date(p.invoice_date)}<br>
        Due Date: ${date(p.due_date)}
      </div>
    </div>

    <table>
      <thead>
        <tr>
          <th>#</th><th>Product</th><th>Batch</th>
          <th>Qty</th><th>Free</th><th>Rate</th>
          <th>Disc.</th><th class="right">Amount</th>
        </tr>
      </thead>
      <tbody>${rows}</tbody>
    </table>

    <div class="totals">
      <div><span>Subtotal</span><strong>${money(p.subtotal)}</strong></div>
      <div><span>Discount</span><strong>${money(p.discount_amount)}</strong></div>
      <div><span>Taxable Amount</span><strong>${money(p.taxable_amount)}</strong></div>
      <div><span>Tax</span><strong>${money(p.tax_amount)}</strong></div>
      <div><span>Other Charges</span><strong>${money(p.other_charges)}</strong></div>
      <div><span>Round Off</span><strong>${money(p.round_off)}</strong></div>
      <div class="grand"><span>Grand Total</span><strong>${money(p.grand_total)}</strong></div>
      <div><span>Paid</span><strong>${money(p.paid_amount)}</strong></div>
      <div><span>Balance</span><strong>${money(Number(p.grand_total) - Number(p.paid_amount))}</strong></div>
    </div>

    ${p.notes ? `<div class="notes"><strong>Notes</strong><p>${escapeHtml(p.notes)}</p></div>` : ''}
    <script>window.onload = () => window.print()<\/script>
  </body>
  </html>`

  const printWindow = window.open('', '_blank', 'width=1000,height=800')

  if (!printWindow) {
    actionError.value = 'Print window was blocked by the browser.'
    return
  }

  printWindow.document.open()
  printWindow.document.write(html)
  printWindow.document.close()
}

function escapeHtml(value: string) {
  return String(value ?? '').replace(/[&<>"']/g, char => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  })[char]!)
}

function printFromDetails() {
  if (selectedPurchase.value) printPurchase(selectedPurchase.value)
}

function closeMenus(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.row-menu-wrap')) openMenuId.value = ''
}

onMounted(() => {
  loadPurchases()
  document.addEventListener('click', closeMenus)
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  document.removeEventListener('click', closeMenus)
})
</script>

<template>
  <main class="purchase-page">
    <!-- Page heading -->
    <header class="page-header">
      <div class="heading-left">
        <div class="heading-icon">
          <FileText :size="21" />
        </div>
        <div>
          <h1>Purchase Register</h1>
          <p>Supplier invoices, stock purchases and payment records</p>
        </div>
      </div>

      <NuxtLink to="/purchases/new" class="primary-btn">
        <Plus :size="17" />
        New Purchase
      </NuxtLink>
    </header>

    <!-- Summary -->
    <section class="summary-grid">
      <article class="summary-card">
        <span>Total Records</span>
        <strong>{{ total }}</strong>
        <small>Matching purchases</small>
      </article>

      <article class="summary-card">
        <span>Drafts</span>
        <strong>{{ draftCount }}</strong>
        <small>On this page</small>
      </article>

      <article class="summary-card">
        <span>Posted</span>
        <strong>{{ postedCount }}</strong>
        <small>On this page</small>
      </article>

      <article class="summary-card">
        <span>Page Purchase Value</span>
        <strong class="value">{{ money(visibleAmount) }}</strong>
        <small>Current page total</small>
      </article>
    </section>

    <!-- Search and filters -->
    <section class="register-panel">
      <div class="toolbar">
        <label class="search-box">
          <Search :size="17" />
          <input
            v-model="search"
            placeholder="Search purchase no., supplier or bill no."
            @input="onSearch"
          />
          <kbd>Search</kbd>
        </label>

        <select v-model="status" class="filter-select">
          <option value="ALL">All Status</option>
          <option value="DRAFT">Draft</option>
          <option value="POSTED">Posted</option>
          <option value="CANCELLED">Cancelled</option>
        </select>

        <button
          class="refresh-btn"
          :disabled="loading"
          title="Refresh register"
          @click="loadPurchases(true)"
        >
          <RefreshCw :size="16" :class="{ spinning: refreshing }" />
        </button>
      </div>

      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button @click="loadPurchases()"><RefreshCw :size="14" /> Retry</button>
      </div>

      <div v-if="actionError" class="error-banner">
        <span>{{ actionError }}</span>
        <button @click="actionError = ''"><X :size="15" /></button>
      </div>

      <!-- Register table -->
      <div class="table-scroll">
        <table class="purchase-table">
          <thead>
            <tr>
              <th class="number-col">Purchase No.</th>
              <th>Invoice Date</th>
              <th>Supplier</th>
              <th>Supplier Bill No.</th>
              <th class="right">Grand Total</th>
              <th class="right">Paid</th>
              <th>Status</th>
              <th class="actions-col">Actions</th>
            </tr>
          </thead>

          <tbody>
            <tr v-for="item in purchases" :key="item.id">
              <td>
                <button class="purchase-number" @click="openDetails(item)">
                  {{ item.purchase_number }}
                </button>
              </td>

              <td class="date-cell">{{ date(item.invoice_date) }}</td>

              <td>
                <div class="supplier-name">{{ item.supplier_name }}</div>
              </td>

              <td class="bill-cell">{{ item.supplier_invoice_number }}</td>

              <td class="right amount-cell">{{ money(item.grand_total) }}</td>

              <td class="right paid-cell">{{ money(item.paid_amount) }}</td>

              <td>
                <span class="status-pill" :class="item.status.toLowerCase()">
                  <i></i>
                  {{ statusLabel(item.status) }}
                </span>
              </td>

              <td>
                <div class="row-actions">
                  <button title="View purchase" @click="openDetails(item)">
                    <Eye :size="16" />
                  </button>

                  <button
                    title="Edit draft"
                    :disabled="item.status !== 'DRAFT'"
                    @click="editPurchase(item)"
                  >
                    <Pencil :size="15" />
                  </button>

                  <button
                    title="Print purchase"
                    @click="openDetails(item)"
                  >
                    <Printer :size="16" />
                  </button>

                  <div class="row-menu-wrap">
                    <button
                      title="More actions"
                      @click.stop="openMenuId = openMenuId === item.id ? '' : item.id"
                    >
                      <MoreHorizontal :size="17" />
                    </button>

                    <div v-if="openMenuId === item.id" class="row-menu">
                      <button
                        v-if="item.status === 'DRAFT'"
                        @click="postTarget = item; openMenuId = ''"
                      >
                        <CheckCircle2 :size="15" /> Post Purchase
                      </button>

                      <button
                        v-if="item.status === 'DRAFT'"
                        class="danger-action"
                        @click="deleteTarget = item; openMenuId = ''"
                      >
                        <Trash2 :size="15" /> Delete Draft
                      </button>

                      <button @click="openDetails(item)">
                        <Eye :size="15" /> View Details
                      </button>
                    </div>
                  </div>
                </div>
              </td>
            </tr>

            <tr v-if="loading">
              <td colspan="8" class="loading-state">
                <LoaderCircle :size="19" class="spinning" />
                Loading purchase register...
              </td>
            </tr>

            <tr v-else-if="!purchases.length">
              <td colspan="8">
                <div class="empty-state">
                  <div class="empty-icon"><FileText :size="25" /></div>
                  <strong>No purchase records found</strong>
                  <span>Try another search or create a new purchase entry.</span>
                  <NuxtLink to="/purchases/new" class="primary-btn">
                    <Plus :size="16" /> Create Purchase
                  </NuxtLink>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      <footer class="table-footer">
        <span>
          Showing <strong>{{ startRecord }}–{{ endRecord }}</strong>
          of <strong>{{ total }}</strong> records
        </span>

        <div class="pagination">
          <button :disabled="page <= 1 || loading" @click="page--">
            <ChevronLeft :size="17" />
          </button>
          <span>Page <strong>{{ page }}</strong> of {{ totalPages }}</span>
          <button :disabled="page >= totalPages || loading" @click="page++">
            <ChevronRight :size="17" />
          </button>
        </div>
      </footer>
    </section>

    <!-- Purchase details drawer -->
    <Teleport to="body">
      <div v-if="detailOpen" class="overlay" @click.self="detailOpen = false">
        <aside class="detail-drawer">
          <header class="drawer-header">
            <div>
              <span class="drawer-eyebrow">PURCHASE DETAILS</span>
              <h2>{{ selectedPurchase?.purchase.purchase_number || 'Purchase' }}</h2>
            </div>
            <button class="close-btn" @click="detailOpen = false"><X :size="19" /></button>
          </header>

          <div v-if="detailLoading" class="drawer-loading">
            <LoaderCircle :size="22" class="spinning" />
            Loading purchase details...
          </div>

          <template v-else-if="selectedPurchase">
            <div class="drawer-content">
              <div class="detail-status-row">
                <span class="status-pill" :class="selectedPurchase.purchase.status.toLowerCase()">
                  <i></i>
                  {{ statusLabel(selectedPurchase.purchase.status) }}
                </span>
                <span>{{ date(selectedPurchase.purchase.invoice_date) }}</span>
              </div>

              <section class="detail-section">
                <h3>Supplier Information</h3>
                <strong>{{ selectedPurchase.purchase.supplier_name }}</strong>
                <div class="detail-meta">
                  Supplier Bill: {{ selectedPurchase.purchase.supplier_invoice_number }}
                </div>
                <div class="detail-meta">
                  Due Date: {{ date(selectedPurchase.purchase.due_date) }}
                </div>
              </section>

              <section class="detail-section">
                <div class="section-title-row">
                  <h3>Items</h3>
                  <span>{{ selectedPurchase.items.length }} lines</span>
                </div>

                <div
                  v-for="item in selectedPurchase.items"
                  :key="item.id"
                  class="detail-item"
                >
                  <div class="detail-item-top">
                    <div>
                      <strong>{{ item.product_name }}</strong>
                      <small>{{ item.sku }} · Batch {{ item.batch_number }}</small>
                    </div>
                    <strong>{{ money(item.total_amount) }}</strong>
                  </div>

                  <div class="detail-item-meta">
                    <span>Qty: {{ item.quantity }}</span>
                    <span>Free: {{ item.free_quantity }}</span>
                    <span>Rate: {{ money(item.purchase_rate) }}</span>
                  </div>
                </div>
              </section>

              <section class="detail-section">
                <h3>Amount Summary</h3>
                <div class="summary-line">
                  <span>Subtotal</span>
                  <strong>{{ money(selectedPurchase.purchase.subtotal) }}</strong>
                </div>
                <div class="summary-line">
                  <span>Discount</span>
                  <strong>{{ money(selectedPurchase.purchase.discount_amount) }}</strong>
                </div>
                <div class="summary-line">
                  <span>Taxable Amount</span>
                  <strong>{{ money(selectedPurchase.purchase.taxable_amount) }}</strong>
                </div>
                <div class="summary-line">
                  <span>GST</span>
                  <strong>{{ money(selectedPurchase.purchase.tax_amount) }}</strong>
                </div>
                <div class="summary-line">
                  <span>Other Charges</span>
                  <strong>{{ money(selectedPurchase.purchase.other_charges) }}</strong>
                </div>
                <div class="summary-line">
                  <span>Round Off</span>
                  <strong>{{ money(selectedPurchase.purchase.round_off) }}</strong>
                </div>
                <div class="summary-line grand-line">
                  <span>Grand Total</span>
                  <strong>{{ money(selectedPurchase.purchase.grand_total) }}</strong>
                </div>
                <div class="summary-line">
                  <span>Paid</span>
                  <strong>{{ money(selectedPurchase.purchase.paid_amount) }}</strong>
                </div>
                <div class="summary-line balance-line">
                  <span>Balance</span>
                  <strong>{{ money(Number(selectedPurchase.purchase.grand_total) - Number(selectedPurchase.purchase.paid_amount)) }}</strong>
                </div>
              </section>

              <section v-if="selectedPurchase.purchase.notes" class="detail-section">
                <h3>Notes</h3>
                <p class="notes-text">{{ selectedPurchase.purchase.notes }}</p>
              </section>
            </div>

            <footer class="drawer-footer">
              <button class="secondary-btn" @click="printFromDetails">
                <Printer :size="16" /> Print
              </button>

              <button
                v-if="selectedPurchase.purchase.status === 'DRAFT'"
                class="secondary-btn"
                @click="editPurchase(selectedPurchase.purchase)"
              >
                <Pencil :size="15" /> Edit
              </button>

              <button
                v-if="selectedPurchase.purchase.status === 'DRAFT'"
                class="primary-btn"
                @click="postTarget = selectedPurchase.purchase; detailOpen = false"
              >
                <CheckCircle2 :size="16" /> Post Purchase
              </button>
            </footer>
          </template>
        </aside>
      </div>
    </Teleport>

    <!-- Delete confirmation -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="overlay confirm-overlay" @click.self="deleteTarget = null">
        <section class="confirm-dialog">
          <div class="confirm-icon danger"><Trash2 :size="21" /></div>
          <h2>Delete this purchase?</h2>
          <p>
            Purchase <strong>{{ deleteTarget.purchase_number }}</strong>
            will be deleted. This action should only be allowed for draft purchases.
          </p>
          <div class="confirm-actions">
            <button class="secondary-btn" @click="deleteTarget = null">Cancel</button>
            <button class="danger-btn" :disabled="deleteLoading" @click="deletePurchase">
              <LoaderCircle v-if="deleteLoading" :size="15" class="spinning" />
              <Trash2 v-else :size="15" />
              Delete Purchase
            </button>
          </div>
        </section>
      </div>
    </Teleport>

    <!-- Post confirmation -->
    <Teleport to="body">
      <div v-if="postTarget" class="overlay confirm-overlay" @click.self="postTarget = null">
        <section class="confirm-dialog">
          <div class="confirm-icon"><CheckCircle2 :size="21" /></div>
          <h2>Post this purchase?</h2>
          <p>
            Posting <strong>{{ postTarget.purchase_number }}</strong>
            will update inventory batches and supplier ledger.
            This action should be treated as final.
          </p>
          <div class="confirm-actions">
            <button class="secondary-btn" @click="postTarget = null">Cancel</button>
            <button class="primary-btn" :disabled="postLoading" @click="postPurchase(postTarget)">
              <LoaderCircle v-if="postLoading" :size="15" class="spinning" />
              <CheckCircle2 v-else :size="15" />
              Confirm Post
            </button>
          </div>
        </section>
      </div>
    </Teleport>
  </main>
</template>

<style scoped>
.purchase-page {
  max-width: 1700px;
  margin: auto;
  padding: 25px 27px 40px;
  color: #171717;
  font-size: 13px;
}

.page-header,.heading-left,.toolbar,.search-box,.table-footer,
.pagination,.row-actions,.detail-status-row,.section-title-row,
.detail-item-top,.drawer-footer,.confirm-actions {
  display:flex;
  align-items:center;
}

.page-header { justify-content:space-between; gap:20px; margin-bottom:23px; }
.heading-left { gap:13px; }
.heading-icon {
  display:grid; place-items:center; width:43px; height:43px;
  border:1px solid #e5e5e5; border-radius:9px; background:#fff;
}
.page-header h1 { margin:0; font-size:24px; font-weight:720; letter-spacing:-.7px; }
.page-header p { margin:5px 0 0; color:#777; font-size:12px; }

.primary-btn,.secondary-btn,.danger-btn {
  display:inline-flex; align-items:center; justify-content:center; gap:8px;
  min-height:38px; padding:0 13px; border-radius:7px;
  font-size:12px; font-weight:650; text-decoration:none; cursor:pointer;
}
.primary-btn { border:1px solid #171717; background:#171717; color:#fff; }
.secondary-btn { border:1px solid #e2e2e2; background:#fff; color:#333; }
.danger-btn { border:1px solid #b42318; background:#b42318; color:#fff; }
button:disabled { opacity:.4; cursor:not-allowed; }

.summary-grid {
  display:grid; grid-template-columns:repeat(4,minmax(0,1fr));
  gap:12px; margin-bottom:17px;
}
.summary-card {
  display:flex; flex-direction:column; gap:7px;
  min-height:102px; padding:15px 16px;
  border:1px solid #e7e7e7; border-radius:9px; background:#fff;
}
.summary-card>span { color:#737373; font-size:11px; font-weight:600; }
.summary-card strong { font-size:22px; font-weight:720; letter-spacing:-.5px; }
.summary-card strong.value { font-size:18px; }
.summary-card small { color:#999; font-size:10px; }

.register-panel { overflow:visible; border:1px solid #e5e5e5; border-radius:10px; background:#fff; }
.toolbar { gap:9px; padding:14px; border-bottom:1px solid #eee; }
.search-box {
  flex:1; min-width:180px; height:39px; gap:9px; padding:0 11px;
  border:1px solid #e5e5e5; border-radius:7px; color:#777;
}
.search-box input { flex:1; min-width:0; border:0; outline:0; color:#171717; font:inherit; font-size:12px; }
.search-box kbd { color:#aaa; font-size:10px; }
.filter-select,.refresh-btn {
  height:39px; border:1px solid #e5e5e5; border-radius:7px;
  background:#fff; color:#444; font:inherit; font-size:12px;
}
.filter-select { min-width:135px; padding:0 10px; }
.refresh-btn { display:grid; place-items:center; width:39px; cursor:pointer; }

.error-banner {
  display:flex; align-items:center; justify-content:space-between; gap:12px;
  margin:12px 14px 0; padding:10px 12px;
  border-radius:6px; background:#fff0ef; color:#a33b3b; font-size:11px;
}
.error-banner button { display:flex; align-items:center; gap:5px; border:0; background:none; color:inherit; cursor:pointer; }

.table-scroll { overflow-x:auto; }
.purchase-table { width:100%; border-collapse:collapse; white-space:nowrap; }
.purchase-table th {
  padding:13px 14px; background:#fafafa; border-bottom:1px solid #eaeaea;
  color:#777; text-align:left; font-size:10px; font-weight:700;
  text-transform:uppercase; letter-spacing:.04em;
}
.purchase-table td { padding:12px 14px; border-bottom:1px solid #f0f0f0; color:#444; font-size:12px; }
.purchase-table tbody tr:hover { background:#fcfcfc; }
.number-col { min-width:145px; }
.actions-col { width:145px; text-align:center!important; }
.right { text-align:right!important; font-variant-numeric:tabular-nums; }
.purchase-number {
  padding:0; border:0; background:none; color:#171717;
  font-size:12px; font-weight:700; cursor:pointer;
}
.purchase-number:hover { text-decoration:underline; }
.supplier-name { max-width:220px; overflow:hidden; text-overflow:ellipsis; font-weight:600; }
.date-cell,.bill-cell,.paid-cell { color:#737373!important; }
.amount-cell { color:#171717!important; font-weight:700; }

.status-pill {
  display:inline-flex; align-items:center; gap:6px; padding:5px 9px;
  border-radius:5px; background:#f3f3f3; color:#555;
  font-size:10px; font-weight:700;
}
.status-pill i { width:6px; height:6px; border-radius:50%; background:#999; }
.status-pill.draft { background:#fff5e8; color:#95621b; }
.status-pill.draft i { background:#c58a2a; }
.status-pill.posted { background:#eaf5ed; color:#24623b; }
.status-pill.posted i { background:#398c55; }
.status-pill.cancelled { background:#fceeee; color:#a33b3b; }
.status-pill.cancelled i { background:#c65b5b; }

.row-actions { justify-content:center; gap:3px; }
.row-actions>button,.row-menu-wrap>button {
  display:grid; place-items:center; width:29px; height:29px;
  border:0; border-radius:5px; background:transparent; color:#737373; cursor:pointer;
}
.row-actions>button:hover,.row-menu-wrap>button:hover { background:#f0f0f0; color:#171717; }
.row-menu-wrap { position:relative; }
.row-menu {
  position:absolute; z-index:20; top:33px; right:0; width:170px;
  padding:5px; border:1px solid #e5e5e5; border-radius:7px;
  background:#fff; box-shadow:0 10px 30px #00000012;
}
.row-menu button {
  display:flex; align-items:center; gap:8px; width:100%;
  padding:9px; border:0; border-radius:5px; background:#fff;
  color:#444; text-align:left; font-size:11px; cursor:pointer;
}
.row-menu button:hover { background:#f5f5f5; }
.row-menu .danger-action { color:#b42318; }

.empty-state {
  display:flex; flex-direction:column; align-items:center; gap:10px;
  padding:55px 15px; color:#888;
}
.empty-icon {
  display:grid; place-items:center; width:49px; height:49px;
  border:1px solid #e8e8e8; border-radius:12px; background:#fafafa;
}
.empty-state strong { color:#222; font-size:14px; }
.empty-state span { margin-bottom:5px; font-size:11px; }
.loading-state {
  display:flex; align-items:center; justify-content:center; gap:10px;
  padding:40px!important; color:#777!important;
}
.table-footer {
  justify-content:space-between; min-height:55px; padding:10px 15px;
  border-top:1px solid #eee; color:#888; font-size:11px;
}
.table-footer strong { color:#333; }
.pagination { gap:12px; }
.pagination button {
  display:grid; place-items:center; width:30px; height:30px;
  border:1px solid #e5e5e5; border-radius:6px; background:#fff; cursor:pointer;
}
.pagination button:disabled { opacity:.35; cursor:not-allowed; }
.pagination span { color:#777; }

.overlay {
  position:fixed; z-index:1000; inset:0; display:flex;
  justify-content:flex-end; background:#17171770; backdrop-filter:blur(2px);
}
.detail-drawer {
  display:flex; flex-direction:column; width:min(570px,100%);
  height:100%; background:#fff; box-shadow:-15px 0 50px #0002;
}
.drawer-header {
  display:flex; align-items:center; justify-content:space-between;
  padding:20px; border-bottom:1px solid #eee;
}
.drawer-eyebrow { color:#888; font-size:9px; font-weight:750; letter-spacing:.12em; }
.drawer-header h2 { margin:5px 0 0; font-size:19px; }
.close-btn {
  display:grid; place-items:center; width:34px; height:34px;
  border:1px solid #eee; border-radius:7px; background:#fff; cursor:pointer;
}
.drawer-content { flex:1; overflow:auto; padding:18px 20px 25px; }
.drawer-loading { display:flex; flex:1; align-items:center; justify-content:center; gap:10px; color:#777; }
.detail-status-row { justify-content:space-between; margin-bottom:20px; color:#777; font-size:11px; }
.detail-section { padding:16px 0; border-top:1px solid #eee; }
.detail-section h3 { margin:0 0 13px; font-size:12px; font-weight:750; }
.detail-section>strong { font-size:14px; }
.detail-meta { margin-top:7px; color:#777; font-size:11px; }
.section-title-row { justify-content:space-between; }
.section-title-row span { color:#888; font-size:10px; }
.detail-item { padding:13px 0; border-top:1px solid #f0f0f0; }
.detail-item-top { justify-content:space-between; gap:10px; }
.detail-item-top>div { display:flex; flex-direction:column; gap:5px; }
.detail-item-top strong { font-size:12px; }
.detail-item-top small { color:#888; font-size:10px; }
.detail-item-meta { display:flex; gap:16px; margin-top:9px; color:#777; font-size:10px; }
.summary-line { display:flex; justify-content:space-between; padding:8px 0; color:#777; font-size:11px; }
.summary-line strong { color:#333; font-variant-numeric:tabular-nums; }
.grand-line { margin-top:5px; padding-top:13px; border-top:1px solid #ddd; color:#171717; font-size:14px; font-weight:700; }
.grand-line strong { color:#171717; font-size:16px; }
.balance-line { color:#171717; font-weight:700; }
.notes-text { margin:0; color:#666; font-size:12px; line-height:1.6; }
.drawer-footer {
  justify-content:flex-end; gap:8px; padding:13px 18px;
  border-top:1px solid #eee; background:#fafafa;
}

.confirm-overlay { align-items:center; justify-content:center; padding:16px; }
.confirm-dialog {
  width:min(410px,100%); padding:24px; border-radius:12px;
  background:#fff; box-shadow:0 20px 70px #0003;
}
.confirm-icon {
  display:grid; place-items:center; width:43px; height:43px;
  border-radius:10px; background:#f1f1f1; color:#333;
}
.confirm-icon.danger { background:#fff0ef; color:#b42318; }
.confirm-dialog h2 { margin:17px 0 8px; font-size:18px; }
.confirm-dialog p { color:#777; font-size:12px; line-height:1.7; }
.confirm-dialog p strong { color:#333; }
.confirm-actions { justify-content:flex-end; gap:8px; margin-top:20px; }

.spinning { animation:spin 1s linear infinite; }
@keyframes spin { to { transform:rotate(360deg); } }

@media(max-width:900px) {
  .summary-grid { grid-template-columns:repeat(2,minmax(0,1fr)); }
}
@media(max-width:650px) {
  .purchase-page { padding:17px 11px 30px; }
  .page-header { align-items:flex-start; }
  .page-header h1 { font-size:20px; }
  .page-header p { max-width:230px; line-height:1.5; }
  .heading-icon { width:37px; height:37px; }
  .primary-btn { padding:0 10px; }
  .summary-grid { gap:8px; }
  .summary-card { min-height:90px; padding:12px; }
  .summary-card strong { font-size:19px; }
  .summary-card strong.value { font-size:15px; }
  .toolbar { flex-wrap:wrap; padding:10px; }
  .search-box { flex-basis:100%; }
  .filter-select { flex:1; }
  .purchase-table { min-width:1000px; }
  .table-footer { align-items:flex-start; flex-direction:column; gap:10px; }
  .pagination { align-self:flex-end; }
  .drawer-footer { flex-wrap:wrap; }
  .drawer-footer>* { flex:1; }
}
</style>
