
<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue'
import {
  ArrowLeft, Plus, Search, X, Trash2, Package,
  Save, ChevronDown, LoaderCircle, CalendarDays,
} from 'lucide-vue-next'

import type {
  Party, Product, ProductInput, Brand, Composition,
} from '~/composables/useApi'

import PartyFormModal from '~/components/parties/PartyFormModal.vue'
import ProductFormModal from '~/components/products/ProductFormModal.vue'

useSeoMeta({ title: 'New Purchase | AdhirajPOS' })

const API = 'http://127.0.0.1:8080/api'
const api = useApi()

type PurchaseLine = {
  key: number
  product_id?: string
  new_product?: Record<string, any>
  product_name: string
  pack: string
  batch_number: string
  manufacturing_date: string
  expiry_date: string
  quantity: number
  free_quantity: number
  purchase_rate: number
  mrp: number
  selling_rate: number
  discount_percent: number
}

const today = new Date().toISOString().slice(0, 10)

const suppliers = ref<Party[]>([])
const products = ref<Product[]>([])
const brands = ref<Brand[]>([])
const compositions = ref<Composition[]>([])

const loading = ref(false)
const saving = ref(false)
const error = ref('')

const supplierModal = ref(false)
const productModal = ref(false)
const itemDropdown = ref(false)

const supplierSearch = ref('')
const productSearch = ref('')
const selectedSupplierId = ref('')

const header = reactive({
  supplier_invoice_number: '',
  invoice_date: today,
  due_date: '',
  tax_mode: 'EXCLUSIVE' as 'INCLUSIVE' | 'EXCLUSIVE',
  tax_type: 'INTRA_STATE' as 'INTRA_STATE' | 'INTER_STATE',
  notes: '',
})

const productForm = reactive<ProductInput>({
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

const brandSearch = ref('')
const compositionSearch = ref('')
const newBrandName = ref('')
const newCompositionName = ref('')
const addingBrand = ref(false)
const addingComposition = ref(false)
const masterLoading = ref(false)
const productSaving = ref(false)
const productError = ref('')

const lines = ref<PurchaseLine[]>([])
let lineKey = 0

const batchModal = ref(false)
const batchProduct = ref<Product | null>(null)
const batchNewProduct = ref<Record<string, any> | null>(null)
const editingLineKey = ref<number | null>(null)

const batch = reactive({
  batch_number: '',
  manufacturing_date: '',
  expiry_date: '',
  quantity: 1,
  free_quantity: 0,
  purchase_rate: 0,
  mrp: 0,
  selling_rate: 0,
  discount_percent: 0,
})

const selectedSupplier = computed(() =>
  suppliers.value.find(s => s.id === selectedSupplierId.value) ?? null,
)

const filteredSuppliers = computed(() => {
  const q = supplierSearch.value.toLowerCase().trim()
  return suppliers.value.filter(s => s.name.toLowerCase().includes(q))
})

const filteredProducts = computed(() => {
  const q = productSearch.value.toLowerCase().trim()
  if (!q) return products.value.slice(0, 30)

  return products.value.filter(p =>
    p.name.toLowerCase().includes(q) ||
    p.sku.toLowerCase().includes(q) ||
    (p.barcode || '').toLowerCase().includes(q),
  ).slice(0, 30)
})

const subtotal = computed(() =>
  lines.value.reduce((sum, line) =>
    sum + line.quantity * line.purchase_rate, 0),
)

const discountTotal = computed(() =>
  lines.value.reduce((sum, line) =>
    sum + line.quantity * line.purchase_rate *
    line.discount_percent / 100, 0),
)

const netBeforeTax = computed(() => subtotal.value - discountTotal.value)

function money(value: number) {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2,
  }).format(Number(value || 0))
}

function lineAmount(line: PurchaseLine) {
  const amount = line.quantity * line.purchase_rate
  return amount - amount * line.discount_percent / 100
}

function formatPack(product: Product) {
  return [product.pack_size, product.unit].filter(Boolean).join(' ') || '—'
}

async function loadMasters() {
  loading.value = true
  error.value = ''

  try {
    const [supplierResult, productResult, brandResult, compositionResult] =
      await Promise.all([
        api.getParties({
          role: 'SUPPLIER',
          active: true,
          page: 1,
          limit: 100,
        }),
        api.getProducts({ page: 1, limit: 100 }),
        api.getBrands({ page: 1, limit: 100 }),
        api.getCompositions({ page: 1, limit: 100 }),
      ])

    suppliers.value = supplierResult.data
    products.value = productResult.data
    brands.value = brandResult.data.filter(x => x.is_active)
    compositions.value = compositionResult.data.filter(x => x.is_active)
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unable to load data.'
  } finally {
    loading.value = false
  }
}

function resetBatch() {
  Object.assign(batch, {
    batch_number: '',
    manufacturing_date: '',
    expiry_date: '',
    quantity: 1,
    free_quantity: 0,
    purchase_rate: 0,
    mrp: 0,
    selling_rate: 0,
    discount_percent: 0,
  })
}

function openBatchForProduct(product: Product) {
  batchProduct.value = product
  batchNewProduct.value = null
  editingLineKey.value = null
  resetBatch()
  batchModal.value = true
  itemDropdown.value = false
  productSearch.value = ''
}

function openBatchForNewProduct(product: Record<string, any>) {
  batchProduct.value = null
  batchNewProduct.value = product
  editingLineKey.value = null
  resetBatch()
  batchModal.value = true
  productModal.value = false
  itemDropdown.value = false
}

function editBatch(line: PurchaseLine) {
  const product = line.product_id
    ? products.value.find(p => p.id === line.product_id) ?? null
    : null

  batchProduct.value = product
  batchNewProduct.value = line.new_product ?? null
  editingLineKey.value = line.key

  Object.assign(batch, {
    batch_number: line.batch_number,
    manufacturing_date: line.manufacturing_date,
    expiry_date: line.expiry_date,
    quantity: line.quantity,
    free_quantity: line.free_quantity,
    purchase_rate: line.purchase_rate,
    mrp: line.mrp,
    selling_rate: line.selling_rate,
    discount_percent: line.discount_percent,
  })

  batchModal.value = true
}

function saveBatch() {
  if (!batch.batch_number.trim()) {
    error.value = 'Batch number is required.'
    return
  }

  if (batch.quantity <= 0) {
    error.value = 'Quantity must be greater than zero.'
    return
  }

  if (
    batch.manufacturing_date &&
    batch.expiry_date &&
    batch.expiry_date < batch.manufacturing_date
  ) {
    error.value = 'Expiry date cannot be earlier than manufacturing date.'
    return
  }

  const product = batchProduct.value
  const newProduct = batchNewProduct.value

  if (!product && !newProduct) return

  const line: PurchaseLine = {
    key: editingLineKey.value ?? ++lineKey,
    product_id: product?.id,
    new_product: newProduct ?? undefined,
    product_name: product?.name ?? newProduct?.name ?? '',
    pack: product ? formatPack(product) :
      [newProduct?.pack_size, newProduct?.unit].filter(Boolean).join(' ') || '—',
    ...batch,
    batch_number: batch.batch_number.trim(),
  }

  if (editingLineKey.value !== null) {
    const index = lines.value.findIndex(x => x.key === editingLineKey.value)
    if (index !== -1) lines.value[index] = line
  } else {
    lines.value.push(line)
  }

  batchModal.value = false
  error.value = ''
  resetBatch()
}

function removeLine(key: number) {
  lines.value = lines.value.filter(line => line.key !== key)
}

function openNewProduct() {
  Object.assign(productForm, {
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

  productError.value = ''
  productModal.value = true
  itemDropdown.value = false
}

async function saveNewProduct() {
  if (!productForm.name.trim() || !productForm.sku.trim()) {
    productError.value = 'Product name and SKU are required.'
    return
  }

  productSaving.value = true
  productError.value = ''

  try {
    const stagedProduct = {
      sku: productForm.sku.trim(),
      name: productForm.name.trim(),
      generic_name: productForm.generic_name || null,
      brand_id: productForm.brand_id || null,
      brand_name: productForm.brand_name || null,
      composition_ids: productForm.composition_ids || [],
      dosage_form: productForm.dosage_form || null,
      strength: productForm.strength || null,
      pack_size: productForm.pack_size || null,
      unit: productForm.unit || 'PCS',
      hsn_code: productForm.hsn_code || null,
      barcode: productForm.barcode || null,
      is_prescription_required: productForm.is_prescription_required,
      schedule_class: productForm.schedule_class,
      is_narcotic: productForm.is_narcotic,
      units_per_pack: productForm.units_per_pack || 1,
    }

    // New product is staged in the purchase. Backend creates it on posting.
    openBatchForNewProduct(stagedProduct)
  } finally {
    productSaving.value = false
  }
}

async function addBrand() {
  const name = newBrandName.value.trim()
  if (!name) return

  addingBrand.value = true
  try {
    const created = await api.createBrand({ name })
    brands.value.push(created)
    productForm.brand_id = created.id
    productForm.brand_name = created.name
    newBrandName.value = ''
  } catch (err) {
    productError.value = err instanceof Error ? err.message : 'Unable to add brand.'
  } finally {
    addingBrand.value = false
  }
}

async function addComposition() {
  const name = newCompositionName.value.trim()
  if (!name) return

  addingComposition.value = true
  try {
    const created = await api.createComposition({ name })
    compositions.value.push(created)
    productForm.composition_ids = [created.id]
    newCompositionName.value = ''
  } catch (err) {
    productError.value = err instanceof Error ? err.message : 'Unable to add composition.'
  } finally {
    addingComposition.value = false
  }
}

async function saveDraft() {
  error.value = ''

  if (!selectedSupplierId.value) {
    error.value = 'Please select a supplier.'
    return
  }

  if (!header.supplier_invoice_number.trim()) {
    error.value = 'Supplier invoice number is required.'
    return
  }

  if (!lines.value.length) {
    error.value = 'Add at least one purchase item.'
    return
  }

  saving.value = true

  try {
    const payload = {
      supplier_id: selectedSupplierId.value,
      supplier_invoice_number: header.supplier_invoice_number.trim(),
      invoice_date: header.invoice_date,
      due_date: header.due_date || null,
      notes: header.notes.trim() || null,
      tax_mode: header.tax_mode,
      tax_type: header.tax_type,
      items: lines.value.map(line => ({
        ...(line.new_product
          ? { new_product: line.new_product }
          : { product_id: line.product_id }),
        batch_number: line.batch_number,
        manufacturing_date: line.manufacturing_date || null,
        expiry_date: line.expiry_date || null,
        quantity: line.quantity,
        free_quantity: line.free_quantity,
        purchase_rate: line.purchase_rate,
        mrp: line.mrp,
        selling_rate: line.selling_rate,
        discount_percent: line.discount_percent,
      })),
    }

    await $fetch(`${API}/purchases`, {
      method: 'POST',
      body: payload,
    })

    await navigateTo('/purchases')
  } catch (err: any) {
    error.value = err?.data?.error || err?.data?.message ||
      err?.message || 'Unable to save purchase.'
  } finally {
    saving.value = false
  }
}

async function supplierSaved() {
  supplierModal.value = false
  await loadMasters()
}

function closeOnEscape(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    batchModal.value = false
    itemDropdown.value = false
  }
}

onMounted(() => {
  loadMasters()
  document.addEventListener('keydown', closeOnEscape)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', closeOnEscape)
})
</script>

<template>
  <main class="purchase-screen">
    <header class="topbar">
      <div class="topbar-title">
        <NuxtLink to="/purchases" class="back-link">
          <ArrowLeft :size="17" />
        </NuxtLink>
        <div>
          <h1>New Purchase</h1>
          <p>Purchase entry</p>
        </div>
      </div>

      <div class="top-actions">
        <NuxtLink to="/purchases" class="btn btn-light">Cancel</NuxtLink>
        <button class="btn btn-dark" :disabled="saving" @click="saveDraft">
          <LoaderCircle v-if="saving" class="spin" :size="16" />
          <Save v-else :size="16" />
          Save Draft
        </button>
      </div>
    </header>

    <div v-if="error" class="error-banner">
      {{ error }}
      <button @click="error = ''"><X :size="15" /></button>
    </div>

    <section class="invoice-header">
      <div class="field supplier-field">
        <label>Supplier</label>
        <div class="supplier-select">
          <select v-model="selectedSupplierId">
            <option value="">Select supplier</option>
            <option v-for="supplier in suppliers" :key="supplier.id" :value="supplier.id">
              {{ supplier.name }}
            </option>
          </select>
          <button class="add-supplier" type="button" @click="supplierModal = true">
            <Plus :size="15" /> New
          </button>
        </div>
        <small v-if="selectedSupplier">
          {{ selectedSupplier.phone || '' }}
          <span v-if="selectedSupplier.gstin"> · GSTIN {{ selectedSupplier.gstin }}</span>
        </small>
      </div>

      <div class="field">
        <label>Supplier Bill No. <b>*</b></label>
        <input v-model="header.supplier_invoice_number" placeholder="Invoice number" />
      </div>

      <div class="field">
        <label>Invoice Date</label>
        <input v-model="header.invoice_date" type="date" />
      </div>

      <div class="field">
        <label>Due Date</label>
        <input v-model="header.due_date" type="date" />
      </div>

      <div class="field">
        <label>Tax Mode</label>
        <select v-model="header.tax_mode">
          <option value="EXCLUSIVE">Exclusive</option>
          <option value="INCLUSIVE">Inclusive</option>
        </select>
      </div>

      <div class="field">
        <label>Supply Type</label>
        <select v-model="header.tax_type">
          <option value="INTRA_STATE">Intra-state</option>
          <option value="INTER_STATE">Inter-state</option>
        </select>
      </div>
    </section>

    <section class="items-panel">
      <div class="items-titlebar">
        <div>
          <h2>Purchase Items</h2>
          <span>{{ lines.length }} item lines</span>
        </div>

        <div class="item-add-wrap">
          <button class="btn btn-dark add-item-btn" @click="itemDropdown = !itemDropdown">
            <Plus :size="16" /> Add Item
            <ChevronDown :size="15" />
          </button>

          <div v-if="itemDropdown" class="product-picker">
            <div class="picker-search">
              <Search :size="16" />
              <input
                v-model="productSearch"
                autofocus
                placeholder="Search product name, SKU or barcode"
              />
              <button @click="itemDropdown = false"><X :size="15" /></button>
            </div>

            <div class="picker-list">
              <button
                v-for="product in filteredProducts"
                :key="product.id"
                class="product-option"
                @click="openBatchForProduct(product)"
              >
                <span class="product-option-icon"><Package :size="17" /></span>
                <span class="product-option-copy">
                  <strong>{{ product.name }}</strong>
                  <small>{{ product.sku }} · {{ formatPack(product) }}</small>
                </span>
                <Plus :size="16" />
              </button>

              <div v-if="!filteredProducts.length" class="picker-empty">
                No matching product
              </div>
            </div>

            <button class="new-product-option" @click="openNewProduct">
              <Plus :size="17" />
              Create new item
            </button>
          </div>
        </div>
      </div>

      <div class="items-table-wrap">
        <table class="items-table">
          <thead>
            <tr>
              <th class="col-no">#</th>
              <th class="col-product">Product</th>
              <th>Pack</th>
              <th>Batch</th>
              <th class="numeric">Qty</th>
              <th class="numeric">Free</th>
              <th class="numeric">P. Rate</th>
              <th class="numeric">Dis %</th>
              <th class="numeric">Amount</th>
              <th></th>
            </tr>
          </thead>

          <tbody>
            <tr v-for="(line, index) in lines" :key="line.key">
              <td class="muted-cell">{{ index + 1 }}</td>
              <td>
                <button class="product-cell" @click="editBatch(line)">
                  <strong>{{ line.product_name }}</strong>
                  <small>{{ line.new_product ? 'New item' : 'Product master' }}</small>
                </button>
              </td>
              <td>{{ line.pack }}</td>
              <td>
                <button class="batch-cell" @click="editBatch(line)">
                  {{ line.batch_number }}
                </button>
              </td>
              <td class="numeric">{{ line.quantity }}</td>
              <td class="numeric">{{ line.free_quantity }}</td>
              <td class="numeric">{{ line.purchase_rate.toFixed(2) }}</td>
              <td class="numeric">{{ line.discount_percent }}%</td>
              <td class="numeric amount-cell">{{ money(lineAmount(line)) }}</td>
              <td>
                <button class="delete-line" title="Remove item" @click="removeLine(line.key)">
                  <Trash2 :size="15" />
                </button>
              </td>
            </tr>

            <tr v-if="!lines.length">
              <td colspan="10">
                <div class="empty-items">
                  <Package :size="27" />
                  <strong>No items added</strong>
                  <span>Use Add Item to select a product or create a new one.</span>
                  <button class="btn btn-light" @click="itemDropdown = true">
                    <Plus :size="15" /> Add first item
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="table-bottom">
        <span>Double-click a product or batch row to edit its batch details.</span>
        <button class="text-add" @click="itemDropdown = true">
          <Plus :size="15" /> Add another item
        </button>
      </div>
    </section>

    <section class="invoice-bottom">
      <div class="field notes-field">
        <label>Notes</label>
        <textarea v-model="header.notes" rows="3" placeholder="Purchase remarks..."></textarea>
      </div>

      <div class="totals">
        <div><span>Subtotal</span><strong>{{ money(subtotal) }}</strong></div>
        <div><span>Discount</span><strong>{{ money(discountTotal) }}</strong></div>
        <div class="grand-total">
          <span>Before GST</span>
          <strong>{{ money(netBeforeTax) }}</strong>
        </div>
        <small>Final GST and invoice total are calculated by the backend.</small>
      </div>
    </section>

    <footer class="bottom-actions">
      <span>{{ lines.length }} item lines · {{ selectedSupplier?.name || 'No supplier selected' }}</span>
      <div>
        <NuxtLink to="/purchases" class="btn btn-light">Cancel</NuxtLink>
        <button class="btn btn-dark" :disabled="saving" @click="saveDraft">
          <Save :size="16" /> Save Draft
        </button>
      </div>
    </footer>

    <!-- Batch detail window -->
    <Teleport to="body">
      <div v-if="batchModal" class="modal-backdrop" @click.self="batchModal = false">
        <section class="batch-window">
          <header class="batch-window-header">
            <div>
              <span>BATCH DETAILS</span>
              <h2>{{ batchProduct?.name || batchNewProduct?.name }}</h2>
            </div>
            <button @click="batchModal = false"><X :size="19" /></button>
          </header>

          <div class="batch-window-body">
            <div class="batch-fields">
              <label class="field">
                <span>Batch Number <b>*</b></span>
                <input v-model="batch.batch_number" autofocus placeholder="Enter batch number" />
              </label>
              <label class="field">
                <span>Manufacturing Date</span>
                <input v-model="batch.manufacturing_date" type="date" />
              </label>
              <label class="field">
                <span>Expiry Date</span>
                <input v-model="batch.expiry_date" type="date" />
              </label>
              <label class="field">
                <span>Purchase Rate <b>*</b></span>
                <input v-model.number="batch.purchase_rate" type="number" min="0" step="0.01" />
              </label>
              <label class="field">
                <span>Quantity <b>*</b></span>
                <input v-model.number="batch.quantity" type="number" min="0.001" step="0.001" />
              </label>
              <label class="field">
                <span>Free Quantity</span>
                <input v-model.number="batch.free_quantity" type="number" min="0" step="0.001" />
              </label>
              <label class="field">
                <span>Discount %</span>
                <input v-model.number="batch.discount_percent" type="number" min="0" max="100" step="0.01" />
              </label>
              <label class="field">
                <span>MRP</span>
                <input v-model.number="batch.mrp" type="number" min="0" step="0.01" />
              </label>
              <label class="field">
                <span>Selling Rate</span>
                <input v-model.number="batch.selling_rate" type="number" min="0" step="0.01" />
              </label>
            </div>

            <div class="batch-preview">
              <span>Line amount</span>
              <strong>{{ money(batch.quantity * batch.purchase_rate * (1 - batch.discount_percent / 100)) }}</strong>
            </div>
          </div>

          <footer class="batch-window-footer">
            <button class="btn btn-light" @click="batchModal = false">Cancel</button>
            <button class="btn btn-dark" @click="saveBatch">
              <Plus :size="16" />
              {{ editingLineKey !== null ? 'Update batch' : 'Add to purchase' }}
            </button>
          </footer>
        </section>
      </div>
    </Teleport>

    <PartyFormModal
      :open="supplierModal"
      :party="null"
      default-role="SUPPLIER"
      @close="supplierModal = false"
      @saved="supplierSaved"
    />

    <ProductFormModal
      :model-value="productModal"
      mode="create"
      :form="productForm"
      :brands="brands"
      :compositions="compositions"
      :brand-search="brandSearch"
      :composition-search="compositionSearch"
      :new-brand-name="newBrandName"
      :new-composition-name="newCompositionName"
      :adding-brand="addingBrand"
      :adding-composition="addingComposition"
      :master-loading="masterLoading"
      :saving="productSaving"
      :error-message="productError"
      @close="productModal = false"
      @save="saveNewProduct"
      @add-brand="addBrand"
      @add-composition="addComposition"
      @update:brand-search="brandSearch = $event"
      @update:composition-search="compositionSearch = $event"
      @update:new-brand-name="newBrandName = $event"
      @update:new-composition-name="newCompositionName = $event"
    />
  </main>
</template>

<style scoped>
.purchase-screen {
  max-width: 1700px;
  margin: auto;
  padding: 22px 26px 35px;
  color: #171717;
  font-size: 13px;
}
.topbar,.topbar-title,.top-actions,.invoice-header,.supplier-select,
.items-titlebar,.table-bottom,.invoice-bottom,.bottom-actions,
.bottom-actions>div,.batch-window-header,.batch-window-footer {
  display:flex;
  align-items:center;
}
.topbar { justify-content:space-between; margin-bottom:22px; }
.topbar-title { gap:13px; }
.back-link {
  display:grid; place-items:center; width:37px; height:37px;
  border:1px solid #e5e5e5; border-radius:8px; color:#404040;
}
.topbar h1 { margin:0; font-size:23px; font-weight:700; letter-spacing:-.6px; }
.topbar p { margin:3px 0 0; color:#737373; font-size:12px; }
.top-actions { gap:9px; }
.btn {
  display:inline-flex; align-items:center; justify-content:center; gap:7px;
  min-height:38px; padding:0 13px; border-radius:7px;
  font-size:12px; font-weight:650; text-decoration:none; cursor:pointer;
}
.btn-dark { color:#fff; background:#171717; border:1px solid #171717; }
.btn-light { color:#404040; background:#fff; border:1px solid #e5e5e5; }
.btn:disabled { opacity:.5; cursor:not-allowed; }

.invoice-header {
  display:grid; grid-template-columns:2fr 1.35fr 1fr 1fr 1fr 1.2fr;
  gap:13px; padding:17px; border:1px solid #e5e5e5;
  border-radius:10px; background:#fff; margin-bottom:17px;
}
.field { display:flex; flex-direction:column; gap:7px; min-width:0; }
.field label,.field>span { font-size:11px; font-weight:650; color:#525252; }
.field b { color:#b42318; }
.field input,.field select,.field textarea {
  width:100%; min-width:0; height:38px; padding:0 10px;
  border:1px solid #e5e5e5; border-radius:6px; outline:none;
  background:#fff; color:#171717; font:inherit; font-size:12px;
}
.field textarea { height:auto; padding:10px; resize:vertical; }
.field input:focus,.field select:focus,.field textarea:focus { border-color:#737373; }
.supplier-select { gap:6px; }
.supplier-select select { flex:1; min-width:0; }
.add-supplier {
  display:flex; align-items:center; gap:3px; height:38px;
  padding:0 8px; border:1px solid #e5e5e5; border-radius:6px;
  background:#fafafa; font-size:11px; cursor:pointer;
}
.supplier-field small { color:#737373; font-size:10px; }

.items-panel { border:1px solid #e5e5e5; border-radius:10px; background:#fff; }
.items-titlebar { justify-content:space-between; padding:15px 17px; border-bottom:1px solid #eee; }
.items-titlebar h2 { margin:0; font-size:15px; font-weight:700; }
.items-titlebar span { display:block; margin-top:4px; color:#737373; font-size:11px; }
.item-add-wrap { position:relative; }
.add-item-btn { min-height:36px; }
.product-picker {
  position:absolute; z-index:30; top:44px; right:0; width:min(410px, calc(100vw - 30px));
  overflow:hidden; border:1px solid #dedede; border-radius:9px;
  background:#fff; box-shadow:0 16px 45px #0000001a;
}
.picker-search { display:flex; align-items:center; gap:9px; padding:11px 12px; border-bottom:1px solid #eee; color:#737373; }
.picker-search input { flex:1; min-width:0; border:0; outline:0; font:inherit; font-size:12px; }
.picker-search button { display:grid; place-items:center; border:0; background:none; color:#737373; }
.picker-list { max-height:300px; overflow:auto; padding:5px; }
.product-option {
  display:flex; align-items:center; gap:10px; width:100%; padding:10px;
  border:0; border-radius:6px; background:white; text-align:left; cursor:pointer;
}
.product-option:hover { background:#f5f5f5; }
.product-option-icon { display:grid; place-items:center; width:32px; height:32px; border-radius:7px; background:#f1f1f1; }
.product-option-copy { display:flex; flex:1; flex-direction:column; gap:3px; min-width:0; }
.product-option-copy strong { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:12px; }
.product-option-copy small { color:#737373; font-size:10px; }
.picker-empty { padding:25px; text-align:center; color:#737373; font-size:12px; }
.new-product-option {
  display:flex; align-items:center; gap:9px; width:100%; padding:13px;
  border:0; border-top:1px solid #eee; background:#fafafa;
  color:#171717; font-size:12px; font-weight:700; cursor:pointer;
}
.new-product-option:hover { background:#f0f0f0; }

.items-table-wrap { overflow-x:auto; }
.items-table { width:100%; border-collapse:collapse; white-space:nowrap; }
.items-table th {
  padding:12px 10px; border-bottom:1px solid #e5e5e5;
  background:#fafafa; color:#737373; text-align:left;
  font-size:10px; font-weight:700; letter-spacing:.035em;
}
.items-table td { padding:11px 10px; border-bottom:1px solid #f0f0f0; font-size:12px; }
.items-table tbody tr:hover { background:#fcfcfc; }
.items-table .numeric { text-align:right; font-variant-numeric:tabular-nums; }
.col-no { width:35px; text-align:center!important; }
.col-product { min-width:220px; }
.product-cell,.batch-cell {
  display:flex; flex-direction:column; gap:3px; padding:0;
  border:0; background:none; color:#171717; text-align:left; cursor:pointer;
}
.product-cell strong { font-size:12px; font-weight:650; }
.product-cell small { color:#888; font-size:10px; }
.batch-cell { color:#404040; }
.batch-cell:hover,.product-cell:hover strong { text-decoration:underline; }
.muted-cell { color:#a3a3a3; text-align:center; }
.amount-cell { font-weight:700; }
.delete-line { display:grid; place-items:center; padding:5px; border:0; background:none; color:#a3a3a3; cursor:pointer; }
.delete-line:hover { color:#b42318; }
.empty-items {
  display:flex; flex-direction:column; align-items:center; gap:9px;
  padding:45px 15px; color:#a3a3a3;
}
.empty-items strong { color:#404040; font-size:13px; }
.empty-items span { color:#888; font-size:11px; text-align:center; }
.table-bottom { justify-content:space-between; padding:11px 16px; color:#888; font-size:10px; }
.text-add { display:flex; align-items:center; gap:5px; border:0; background:none; color:#404040; font-size:11px; font-weight:650; cursor:pointer; }

.invoice-bottom {
  justify-content:space-between; align-items:flex-start; gap:35px;
  margin-top:17px; padding:17px; border:1px solid #e5e5e5;
  border-radius:10px; background:white;
}
.notes-field { flex:1; max-width:600px; }
.totals { width:300px; display:flex; flex-direction:column; gap:12px; }
.totals>div { display:flex; justify-content:space-between; color:#737373; font-size:12px; }
.totals strong { color:#171717; font-variant-numeric:tabular-nums; }
.totals .grand-total { padding-top:12px; border-top:1px solid #eee; color:#171717; font-size:14px; font-weight:700; }
.totals small { color:#888; font-size:10px; line-height:1.5; }
.bottom-actions { justify-content:space-between; margin-top:17px; color:#737373; font-size:11px; }
.bottom-actions>div { gap:9px; }

.error-banner {
  display:flex; justify-content:space-between; gap:12px; margin-bottom:13px;
  padding:11px 13px; border-radius:7px; background:#fceeee;
  color:#a33b3b; font-size:12px;
}
.error-banner button { border:0; background:none; color:inherit; }

.modal-backdrop {
  position:fixed; z-index:1000; inset:0; display:grid; place-items:center;
  padding:18px; background:#17171780; backdrop-filter:blur(3px);
}
.batch-window {
  width:min(700px, 100%); overflow:hidden; border:1px solid #e5e5e5;
  border-radius:12px; background:#fff; box-shadow:0 25px 80px #0003;
}
.batch-window-header {
  justify-content:space-between; padding:17px 20px;
  border-bottom:1px solid #eee;
}
.batch-window-header span { color:#737373; font-size:9px; font-weight:750; letter-spacing:.12em; }
.batch-window-header h2 { margin:5px 0 0; font-size:17px; font-weight:700; }
.batch-window-header>button { display:grid; place-items:center; width:33px; height:33px; border:1px solid #eee; border-radius:7px; background:#fff; }
.batch-window-body { padding:20px; }
.batch-fields { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:16px; }
.batch-preview {
  display:flex; justify-content:space-between; align-items:center;
  margin-top:20px; padding:13px 15px; border-radius:7px; background:#f7f7f7;
  color:#737373; font-size:12px;
}
.batch-preview strong { color:#171717; font-size:17px; font-variant-numeric:tabular-nums; }
.batch-window-footer {
  justify-content:flex-end; gap:9px; padding:14px 20px;
  border-top:1px solid #eee; background:#fafafa;
}
.spin { animation:spin 1s linear infinite; }
@keyframes spin { to { transform:rotate(360deg); } }

@media(max-width:1000px) {
  .invoice-header { grid-template-columns:repeat(3,minmax(0,1fr)); }
}
@media(max-width:650px) {
  .purchase-screen { padding:15px 10px 25px; }
  .topbar { align-items:flex-start; }
  .topbar h1 { font-size:19px; }
  .top-actions { gap:5px; }
  .btn { padding:0 9px; }
  .invoice-header { grid-template-columns:repeat(2,minmax(0,1fr)); padding:12px; gap:12px; }
  .supplier-field { grid-column:span 2; }
  .items-titlebar { padding:12px; }
  .items-table { min-width:900px; }
  .table-bottom { align-items:flex-start; gap:10px; }
  .invoice-bottom { flex-direction:column; gap:20px; padding:13px; }
  .notes-field,.totals { width:100%; max-width:none; }
  .bottom-actions { align-items:flex-start; flex-direction:column; gap:12px; }
  .bottom-actions>div { width:100%; }
  .bottom-actions>div>* { flex:1; }
  .batch-fields { grid-template-columns:repeat(2,minmax(0,1fr)); gap:12px; }
  .batch-window-body { padding:14px; }
}
</style>
