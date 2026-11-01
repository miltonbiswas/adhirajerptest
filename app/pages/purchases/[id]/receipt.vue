
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Printer, ArrowLeft, RefreshCw } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const API = 'http://127.0.0.1:8080/api'

interface BusinessProfile {
  business_name: string
  legal_name: string
  business_type: string
  owner_name: string
  phone: string
  email: string
  address_line1: string
  address_line2: string
  city: string
  district: string
  state: string
  pincode: string
  country: string
  gstin: string
  drug_license_retail: string
  drug_license_wholesale: string
  pharmacy_registration: string
}

interface Purchase {
  id: string
  purchase_number: string
  supplier_id: string
  supplier_name: string
  supplier_invoice_number: string
  invoice_date: string
  due_date: string | null
  status: string
  tax_mode: string
  tax_type: string
  subtotal: number
  discount_amount: number
  taxable_amount: number
  tax_amount: number
  other_charges: number
  round_off: number
  grand_total: number
  paid_amount: number
  notes: string | null
  created_at: string
}

interface PurchaseItem {
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

interface Supplier {
  id: string
  name: string
  contact_person?: string
  phone?: string
  alternate_phone?: string
  email?: string
  gstin?: string
  pan?: string
  drug_license_number?: string
  billing_address?: string
  shipping_address?: string
  city?: string
  district?: string
  state?: string
  pincode?: string
}

const loading = ref(true)
const error = ref('')
const purchase = ref<Purchase | null>(null)
const items = ref<PurchaseItem[]>([])
const business = ref<BusinessProfile | null>(null)
const supplier = ref<Supplier | null>(null)

const isDemo = computed(() => route.query.demo === '1')

const demoPurchase: Purchase = {
  id: 'demo',
  purchase_number: 'PUR-2026-0001',
  supplier_id: 'demo-supplier',
  supplier_name: 'ABC Pharma Distributors',
  supplier_invoice_number: 'ABC/INV/2026/1042',
  invoice_date: '2026-11-01',
  due_date: '2026-11-15',
  status: 'POSTED',
  tax_mode: 'EXCLUSIVE',
  tax_type: 'INTRA_STATE',
  subtotal: 2500,
  discount_amount: 125,
  taxable_amount: 2375,
  tax_amount: 118.75,
  other_charges: 0,
  round_off: 0,
  grand_total: 2493.75,
  paid_amount: 0,
  notes: 'Dummy purchase record for receipt layout preview.',
  created_at: '2026-11-01T10:30:00Z',
}

const demoItems: PurchaseItem[] = [
  {
    id: 'demo-item-1',
    line_number: 1,
    product_id: null,
    product_name: 'Paracetamol 500 mg Tablets',
    sku: 'PCM-500',
    batch_number: 'PCM26101',
    manufacturing_date: '2026-08-01',
    expiry_date: '2028-07-31',
    quantity: 100,
    free_quantity: 10,
    purchase_rate: 10,
    mrp: 15,
    selling_rate: 13,
    discount_percent: 5,
    discount_amount: 50,
    gst_rate: 5,
    taxable_amount: 950,
    cgst_amount: 23.75,
    sgst_amount: 23.75,
    igst_amount: 0,
    total_amount: 997.5,
  },
  {
    id: 'demo-item-2',
    line_number: 2,
    product_id: null,
    product_name: 'Cetirizine 10 mg Tablets',
    sku: 'CTZ-010',
    batch_number: 'CTZ26115',
    manufacturing_date: '2026-07-01',
    expiry_date: '2028-06-30',
    quantity: 100,
    free_quantity: 0,
    purchase_rate: 15,
    mrp: 20,
    selling_rate: 18,
    discount_percent: 5,
    discount_amount: 75,
    gst_rate: 5,
    taxable_amount: 1425,
    cgst_amount: 35.625,
    sgst_amount: 35.625,
    igst_amount: 0,
    total_amount: 1496.25,
  },
]

function emptyBusiness(): BusinessProfile {
  return {
    business_name: '',
    legal_name: '',
    business_type: '',
    owner_name: '',
    phone: '',
    email: '',
    address_line1: '',
    address_line2: '',
    city: '',
    district: '',
    state: '',
    pincode: '',
    country: 'India',
    gstin: '',
    drug_license_retail: '',
    drug_license_wholesale: '',
    pharmacy_registration: '',
  }
}

function money(value: unknown) {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value || 0))
}

function number(value: unknown) {
  return Number(value || 0)
}

function date(value?: string | null) {
  if (!value) return '—'
  const d = new Date(`${value.slice(0, 10)}T00:00:00`)
  if (Number.isNaN(d.getTime())) return value
  return d.toLocaleDateString('en-IN', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  })
}

function amountWords(value: number) {
  const ones = [
    '', 'One', 'Two', 'Three', 'Four', 'Five', 'Six', 'Seven',
    'Eight', 'Nine', 'Ten', 'Eleven', 'Twelve', 'Thirteen',
    'Fourteen', 'Fifteen', 'Sixteen', 'Seventeen', 'Eighteen',
    'Nineteen',
  ]
  const tens = [
    '', '', 'Twenty', 'Thirty', 'Forty', 'Fifty',
    'Sixty', 'Seventy', 'Eighty', 'Ninety',
  ]

  function underThousand(n: number): string {
    let result = ''
    if (n >= 100) {
      result += `${ones[Math.floor(n / 100)]} Hundred `
      n %= 100
    }
    if (n >= 20) {
      result += `${tens[Math.floor(n / 10)]} `
      n %= 10
    }
    if (n > 0) result += `${ones[n]} `
    return result.trim()
  }

  function convert(n: number): string {
    if (n === 0) return 'Zero'
    const parts: string[] = []

    const crore = Math.floor(n / 10000000)
    n %= 10000000
    const lakh = Math.floor(n / 100000)
    n %= 100000
    const thousand = Math.floor(n / 1000)
    n %= 1000

    if (crore) parts.push(`${underThousand(crore)} Crore`)
    if (lakh) parts.push(`${underThousand(lakh)} Lakh`)
    if (thousand) parts.push(`${underThousand(thousand)} Thousand`)
    if (n) parts.push(underThousand(n))

    return parts.join(' ')
  }

  const rupees = Math.floor(value)
  const paise = Math.round((value - rupees) * 100)

  return `${convert(rupees)} Rupees${paise ? ` and ${convert(paise)} Paise` : ''} Only`
}

const totalUnits = computed(() =>
  items.value.reduce((sum, item) =>
    sum + number(item.quantity) + number(item.free_quantity), 0),
)

const totalCgst = computed(() =>
  items.value.reduce((sum, item) => sum + number(item.cgst_amount), 0),
)

const totalSgst = computed(() =>
  items.value.reduce((sum, item) => sum + number(item.sgst_amount), 0),
)

const totalIgst = computed(() =>
  items.value.reduce((sum, item) => sum + number(item.igst_amount), 0),
)

const totalDiscount = computed(() =>
  items.value.reduce((sum, item) => sum + number(item.discount_amount), 0),
)

const businessAddress = computed(() => {
  if (!business.value) return ''
  return [
    business.value.address_line1,
    business.value.address_line2,
    business.value.city,
    business.value.district,
    business.value.state,
    business.value.pincode,
  ].filter(Boolean).join(', ')
})

const supplierAddress = computed(() => {
  if (!supplier.value) return ''
  return [
    supplier.value.billing_address,
    supplier.value.city,
    supplier.value.district,
    supplier.value.state,
    supplier.value.pincode,
  ].filter(Boolean).join(', ')
})

async function loadReceipt() {
  loading.value = true
  error.value = ''

  try {
    const [profileResult] = await Promise.all([
      $fetch<Partial<BusinessProfile>>(`${API}/business-profile`)
        .catch(() => null),
    ])

    business.value = Object.assign(emptyBusiness(), profileResult || {})

    if (isDemo.value) {
      purchase.value = demoPurchase
      items.value = demoItems
      supplier.value = {
        id: 'demo-supplier',
        name: 'ABC Pharma Distributors',
        contact_person: 'Purchase Department',
        phone: '9876543210',
        gstin: '08ABCDE1234F1Z5',
        drug_license_number: 'DL-DEMO-2026',
        billing_address: 'Industrial Area, Main Road',
        city: 'Sirohi',
        district: 'Sirohi',
        state: 'Rajasthan',
        pincode: '307001',
      }
      return
    }

    const id = String(route.params.id || '')

    if (!id || id === 'undefined') {
      throw new Error('Purchase ID is missing.')
    }

    const result = await $fetch<{
      purchase: Purchase
      items: PurchaseItem[]
    }>(`${API}/purchases/${id}`)

    purchase.value = result.purchase
    items.value = result.items || []

    // Supplier details are loaded separately from the Parties API.
    const parties = await $fetch<any>(`${API}/parties`, {
      query: { search: result.purchase.supplier_name, page: 1, limit: 20 },
    }).catch(() => null)

    const partyRows = parties?.data || parties?.parties || []
    supplier.value = partyRows.find(
      (party: Supplier) => party.id === result.purchase.supplier_id,
    ) || null
  } catch (e: any) {
    error.value =
      e?.data?.message ||
      e?.data?.error ||
      e?.message ||
      'Unable to load purchase receipt.'
  } finally {
    loading.value = false
  }
}

function printReceipt() {
  window.print()
}

onMounted(loadReceipt)
</script>

<template>
  <main class="receipt-page">
    <div class="screen-toolbar no-print">
      <button class="toolbar-button" @click="router.back()">
        <ArrowLeft :size="17" />
        Back
      </button>

      <div class="toolbar-right">
        <span v-if="isDemo" class="demo-label">DEMO PREVIEW — NOT A SAVED RECORD</span>
        <button class="toolbar-button" :disabled="loading" @click="loadReceipt">
          <RefreshCw :size="16" /> Refresh
        </button>
        <button class="print-button" :disabled="loading || !purchase" @click="printReceipt">
          <Printer :size="17" /> Print receipt
        </button>
      </div>
    </div>

    <div v-if="loading" class="page-state no-print">
      Loading purchase receipt…
    </div>

    <div v-else-if="error" class="page-state error-state no-print">
      <strong>Receipt could not be loaded</strong>
      <p>{{ error }}</p>
      <button class="toolbar-button" @click="loadReceipt">Try again</button>
    </div>

    <article v-else-if="purchase" class="receipt-sheet">
      <div v-if="isDemo" class="demo-watermark">DEMO COPY</div>

      <header class="receipt-header">
        <div class="business-brand">
          <div class="business-mark">A</div>
          <div>
            <h1>{{ business?.business_name || 'BUSINESS NAME NOT CONFIGURED' }}</h1>
            <div v-if="business?.legal_name && business.legal_name !== business.business_name">
              {{ business.legal_name }}
            </div>
            <p>{{ businessAddress || 'Business address not configured' }}</p>
            <p v-if="business?.phone">Phone: {{ business.phone }}</p>
            <p v-if="business?.email">Email: {{ business.email }}</p>
            <p v-if="business?.gstin">GSTIN: {{ business.gstin }}</p>
          </div>
        </div>

        <div class="document-heading">
          <div class="document-kicker">INVENTORY &amp; PROCUREMENT</div>
          <h2>PURCHASE RECEIPT</h2>
          <p>GOODS RECEIVED NOTE</p>
          <span class="status-stamp">{{ purchase.status }}</span>
        </div>
      </header>

      <div class="document-rule"></div>

      <section class="reference-grid">
        <div class="reference-block">
          <span class="block-label">PURCHASE DETAILS</span>
          <div class="reference-row">
            <span>Purchase No.</span>
            <strong>{{ purchase.purchase_number }}</strong>
          </div>
          <div class="reference-row">
            <span>Purchase Date</span>
            <strong>{{ date(purchase.invoice_date) }}</strong>
          </div>
          <div class="reference-row">
            <span>Due Date</span>
            <strong>{{ date(purchase.due_date) }}</strong>
          </div>
          <div class="reference-row">
            <span>Created At</span>
            <strong>{{ new Date(purchase.created_at).toLocaleString('en-IN') }}</strong>
          </div>
        </div>

        <div class="reference-block">
          <span class="block-label">SUPPLIER DETAILS</span>
          <h3>{{ purchase.supplier_name }}</h3>
          <p>{{ supplierAddress || 'Supplier address not available' }}</p>
          <p v-if="supplier?.contact_person">Contact: {{ supplier.contact_person }}</p>
          <p v-if="supplier?.phone">Phone: {{ supplier.phone }}</p>
          <p v-if="supplier?.gstin">GSTIN: {{ supplier.gstin }}</p>
          <p v-if="supplier?.drug_license_number">
            Drug Licence: {{ supplier.drug_license_number }}
          </p>
          <div class="reference-row supplier-invoice">
            <span>Supplier Invoice No.</span>
            <strong>{{ purchase.supplier_invoice_number }}</strong>
          </div>
        </div>
      </section>

      <section class="items-section">
        <div class="section-heading">
          <h3>ITEM &amp; BATCH DETAILS</h3>
          <span>{{ items.length }} line items · {{ totalUnits }} total units</span>
        </div>

        <table class="receipt-table">
          <thead>
            <tr>
              <th>#</th>
              <th>Product description</th>
              <th>Batch no.</th>
              <th>MFG</th>
              <th>Expiry</th>
              <th class="num">Qty</th>
              <th class="num">Free</th>
              <th class="num">PTR</th>
              <th class="num">MRP</th>
              <th class="num">Disc.</th>
              <th class="num">GST</th>
              <th class="num">Amount</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, index) in items" :key="item.id">
              <td>{{ item.line_number || index + 1 }}</td>
              <td class="product-description">
                <strong>{{ item.product_name }}</strong>
                <small>SKU: {{ item.sku }}</small>
              </td>
              <td>{{ item.batch_number }}</td>
              <td>{{ date(item.manufacturing_date) }}</td>
              <td>{{ date(item.expiry_date) }}</td>
              <td class="num">{{ item.quantity }}</td>
              <td class="num">{{ item.free_quantity }}</td>
              <td class="num">{{ money(item.purchase_rate) }}</td>
              <td class="num">{{ money(item.mrp) }}</td>
              <td class="num">{{ number(item.discount_percent) }}%</td>
              <td class="num">{{ number(item.gst_rate) }}%</td>
              <td class="num amount">{{ money(item.total_amount) }}</td>
            </tr>
            <tr v-if="!items.length">
              <td colspan="12" class="empty-row">No purchase items found.</td>
            </tr>
          </tbody>
        </table>
      </section>

      <section class="bottom-grid">
        <div class="tax-details">
          <h3>TAX SUMMARY</h3>
          <table class="tax-table">
            <thead>
              <tr>
                <th>Tax component</th>
                <th class="num">Amount</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="totalCgst > 0">
                <td>CGST</td>
                <td class="num">{{ money(totalCgst) }}</td>
              </tr>
              <tr v-if="totalSgst > 0">
                <td>SGST</td>
                <td class="num">{{ money(totalSgst) }}</td>
              </tr>
              <tr v-if="totalIgst > 0">
                <td>IGST</td>
                <td class="num">{{ money(totalIgst) }}</td>
              </tr>
              <tr v-if="!totalCgst && !totalSgst && !totalIgst">
                <td colspan="2">No GST amount recorded.</td>
              </tr>
            </tbody>
          </table>

          <div class="receipt-note">
            <strong>Notes</strong>
            <p>{{ purchase.notes || 'No additional notes.' }}</p>
          </div>
        </div>

        <div class="amount-summary">
          <h3>AMOUNT SUMMARY</h3>
          <div class="amount-row">
            <span>Subtotal</span>
            <strong>{{ money(purchase.subtotal) }}</strong>
          </div>
          <div class="amount-row">
            <span>Discount</span>
            <strong>− {{ money(purchase.discount_amount) }}</strong>
          </div>
          <div class="amount-row">
            <span>Taxable amount</span>
            <strong>{{ money(purchase.taxable_amount) }}</strong>
          </div>
          <div class="amount-row">
            <span>Total GST</span>
            <strong>{{ money(purchase.tax_amount) }}</strong>
          </div>
          <div class="amount-row">
            <span>Other charges</span>
            <strong>{{ money(purchase.other_charges) }}</strong>
          </div>
          <div class="amount-row">
            <span>Round off</span>
            <strong>{{ money(purchase.round_off) }}</strong>
          </div>
          <div class="grand-total">
            <span>GRAND TOTAL</span>
            <strong>{{ money(purchase.grand_total) }}</strong>
          </div>
          <div class="amount-row paid-row">
            <span>Paid amount</span>
            <strong>{{ money(purchase.paid_amount) }}</strong>
          </div>
          <div class="amount-row">
            <span>Balance</span>
            <strong>{{ money(number(purchase.grand_total) - number(purchase.paid_amount)) }}</strong>
          </div>
        </div>
      </section>

      <div class="amount-words">
        <strong>Amount in words:</strong>
        {{ amountWords(number(purchase.grand_total)) }}
      </div>

      <section class="receiving-section">
        <div class="receiving-note">
          <h3>GOODS RECEIPT DECLARATION</h3>
          <p>
            The above goods have been recorded against the supplier invoice
            referenced in this document. Batch and expiry details are recorded
            for inventory verification.
          </p>
        </div>

        <div class="signature-grid">
          <div>
            <span>Prepared by</span>
            <div class="signature-line"></div>
            <small>Name / Signature</small>
          </div>
          <div>
            <span>Goods received &amp; checked by</span>
            <div class="signature-line"></div>
            <small>Name / Signature</small>
          </div>
          <div>
            <span>Authorised signatory</span>
            <div class="signature-line"></div>
            <small>Signature / Store stamp</small>
          </div>
        </div>
      </section>

      <footer class="receipt-footer">
        <span>Generated by AdhirajERP</span>
        <span>This is an internal purchase receipt, not a supplier tax invoice.</span>
      </footer>
    </article>
  </main>
</template>

<style>
.receipt-page {
  min-height: 100vh;
  padding: 24px;
  background: #f1f4f2;
  color: #202824;
  font-family: Arial, Helvetica, sans-serif;
}

.screen-toolbar {
  max-width: 1100px;
  margin: 0 auto 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 9px;
}
.toolbar-button,
.print-button {
  min-height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 14px;
  border: 1px solid #d7e1dc;
  border-radius: 7px;
  background: white;
  color: #35423b;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
.print-button {
  border-color: #238d7c;
  background: #238d7c;
  color: white;
}
.toolbar-button:disabled,
.print-button:disabled {
  opacity: .5;
  cursor: not-allowed;
}
.demo-label {
  color: #986b1e;
  font-size: 10px;
  font-weight: 800;
}
.page-state {
  max-width: 1100px;
  margin: 50px auto;
  padding: 35px;
  border: 1px solid #dce5e0;
  border-radius: 9px;
  background: white;
  text-align: center;
}
.error-state { color: #a3342d; }
.error-state p { font-size: 13px; }

.receipt-sheet {
  position: relative;
  width: 210mm;
  min-height: 297mm;
  margin: 0 auto;
  padding: 13mm 13mm 11mm;
  overflow: hidden;
  border: 1px solid #e0e6e2;
  background: #fff;
  box-shadow: 0 8px 35px #17251d12;
  font-size: 10px;
}
.demo-watermark {
  position: absolute;
  top: 46%;
  left: 12%;
  z-index: 0;
  transform: rotate(-30deg);
  color: #238d7c0c;
  font-size: 80px;
  font-weight: 900;
  letter-spacing: 5px;
  pointer-events: none;
}
.receipt-header,
.reference-grid,
.items-section,
.bottom-grid,
.amount-words,
.receiving-section,
.receipt-footer {
  position: relative;
  z-index: 1;
}
.receipt-header {
  display: flex;
  justify-content: space-between;
  gap: 20px;
  align-items: flex-start;
}
.business-brand {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  max-width: 62%;
}
.business-mark {
  width: 43px;
  height: 43px;
  flex: 0 0 43px;
  display: grid;
  place-items: center;
  border: 2px solid #238d7c;
  border-radius: 8px;
  color: #238d7c;
  font-size: 23px;
  font-weight: 900;
}
.business-brand h1 {
  margin: 0 0 5px;
  color: #18231d;
  font-size: 18px;
  font-weight: 850;
  text-transform: uppercase;
}
.business-brand p,
.business-brand div:not(.business-mark) {
  margin: 3px 0;
  color: #56635b;
  font-size: 9px;
  line-height: 1.45;
}
.document-heading {
  min-width: 190px;
  text-align: right;
}
.document-kicker {
  color: #238d7c;
  font-size: 8px;
  font-weight: 800;
  letter-spacing: 1px;
}
.document-heading h2 {
  margin: 7px 0 3px;
  font-size: 19px;
  font-weight: 900;
  letter-spacing: .3px;
}
.document-heading p {
  margin: 0;
  color: #6d7972;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 1px;
}
.status-stamp {
  display: inline-block;
  margin-top: 9px;
  padding: 5px 10px;
  border: 1px solid #238d7c;
  border-radius: 3px;
  color: #238d7c;
  font-size: 9px;
  font-weight: 900;
}
.document-rule {
  height: 2px;
  margin: 17px 0 15px;
  background: #238d7c;
}
.reference-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 13px;
}
.reference-block {
  min-height: 130px;
  padding: 11px 12px;
  border: 1px solid #dce5e0;
  border-radius: 4px;
}
.block-label {
  display: block;
  margin-bottom: 9px;
  color: #238d7c;
  font-size: 8px;
  font-weight: 900;
  letter-spacing: .7px;
}
.reference-row {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  padding: 4px 0;
  color: #67736b;
  font-size: 9px;
}
.reference-row strong {
  color: #28342d;
  text-align: right;
  font-size: 9px;
}
.reference-block h3 {
  margin: 0 0 5px;
  font-size: 12px;
  font-weight: 800;
}
.reference-block p {
  margin: 3px 0;
  color: #606d65;
  font-size: 9px;
  line-height: 1.4;
}
.supplier-invoice {
  margin-top: 5px;
  padding-top: 7px;
  border-top: 1px solid #e9eeeb;
}
.section-heading {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin: 19px 0 9px;
}
.section-heading h3,
.tax-details h3,
.amount-summary h3,
.receiving-note h3 {
  margin: 0;
  font-size: 10px;
  font-weight: 900;
  letter-spacing: .4px;
}
.section-heading span {
  color: #748078;
  font-size: 9px;
}
.receipt-table,
.tax-table {
  width: 100%;
  border-collapse: collapse;
}
.receipt-table {
  table-layout: auto;
}
.receipt-table th {
  padding: 8px 5px;
  border-top: 1px solid #bfcfc7;
  border-bottom: 1px solid #bfcfc7;
  background: #f5f8f6;
  color: #45534b;
  font-size: 8px;
  font-weight: 850;
  text-align: left;
  white-space: nowrap;
}
.receipt-table td {
  padding: 8px 5px;
  border-bottom: 1px solid #e7ece9;
  color: #3d4942;
  font-size: 8.5px;
  vertical-align: top;
}
.receipt-table .num,
.tax-table .num { text-align: right; }
.product-description strong {
  display: block;
  min-width: 105px;
  color: #26342c;
  font-size: 9px;
}
.product-description small {
  display: block;
  margin-top: 3px;
  color: #7d8982;
  font-size: 8px;
}
.amount {
  color: #202b24 !important;
  font-weight: 800;
  white-space: nowrap;
}
.empty-row { padding: 25px !important; text-align: center; }

.bottom-grid {
  display: grid;
  grid-template-columns: 1fr .9fr;
  gap: 18px;
  margin-top: 19px;
}
.tax-details,
.amount-summary {
  min-width: 0;
}
.tax-details h3,
.amount-summary h3 {
  padding-bottom: 8px;
  border-bottom: 1px solid #dce5e0;
}
.tax-table th,
.tax-table td {
  padding: 8px 5px;
  border-bottom: 1px solid #e9eeeb;
  color: #58655d;
  font-size: 9px;
  text-align: left;
}
.tax-table th {
  color: #344139;
  font-weight: 800;
}
.receipt-note {
  margin-top: 18px;
  padding: 10px;
  border: 1px solid #e2eae5;
  border-radius: 4px;
}
.receipt-note strong {
  font-size: 9px;
}
.receipt-note p {
  margin: 6px 0 0;
  color: #68756d;
  font-size: 9px;
  line-height: 1.5;
}
.amount-row {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  padding: 7px 0;
  border-bottom: 1px solid #edf0ee;
  color: #68756d;
  font-size: 9px;
}
.amount-row strong {
  color: #2d3931;
  font-size: 9px;
  text-align: right;
}
.grand-total {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 8px;
  padding: 11px 9px;
  background: #edf5f0;
  color: #176e5d;
  font-size: 10px;
  font-weight: 900;
}
.grand-total strong {
  font-size: 15px;
  white-space: nowrap;
}
.paid-row { margin-top: 6px; }
.amount-words {
  margin-top: 17px;
  padding: 10px;
  border: 1px solid #dfe7e2;
  background: #fafcfb;
  color: #45534b;
  font-size: 9px;
  line-height: 1.5;
}
.amount-words strong { color: #26342c; }

.receiving-section { margin-top: 22px; }
.receiving-note {
  padding: 10px 11px;
  border-left: 3px solid #238d7c;
  background: #f7faf8;
}
.receiving-note p {
  margin: 6px 0 0;
  color: #657169;
  font-size: 9px;
  line-height: 1.5;
}
.signature-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 22px;
  margin-top: 35px;
}
.signature-grid > div {
  display: flex;
  flex-direction: column;
  min-height: 70px;
  color: #354239;
  font-size: 9px;
  font-weight: 750;
}
.signature-line {
  height: 32px;
  margin-top: auto;
  border-bottom: 1px solid #68756d;
}
.signature-grid small {
  margin-top: 5px;
  color: #7c8880;
  font-size: 8px;
  font-weight: 400;
}
.receipt-footer {
  display: flex;
  justify-content: space-between;
  gap: 15px;
  margin-top: 23px;
  padding-top: 9px;
  border-top: 1px solid #dce5e0;
  color: #87928b;
  font-size: 8px;
}

@page {
  size: A4 portrait;
  margin: 0;
}

@media print {
  html, body {
    width: 210mm !important;
    margin: 0 !important;
    padding: 0 !important;
    background: white !important;
  }
  .receipt-page {
    min-height: 0;
    margin: 0;
    padding: 0;
    background: white;
  }
  .no-print { display: none !important; }
  .receipt-sheet {
    width: 210mm;
    min-height: 297mm;
    margin: 0;
    padding: 12mm 12mm 10mm;
    border: 0;
    box-shadow: none;
    print-color-adjust: exact;
    -webkit-print-color-adjust: exact;
  }
  .demo-watermark { color: #238d7c14; }
}
</style>
