const API_BASE_URL = 'http://127.0.0.1:8080/api'

// ─── Shared Types ────────────────────────────────────────────

export interface ListResponse<T> {
  data: T[]
  page: number
  limit: number
  total: number
}

// ─── Products ────────────────────────────────────────────────

export type MedicineSchedule = 'OTC' | 'H' | 'H1' | 'X'

export interface Product {
  id: string
  sku: string
  name: string
  generic_name: string | null
  brand_id: string | null
  brand_name: string | null
  composition_ids: string[]
  composition_names: string[]
  dosage_form: string | null
  strength: string | null
  pack_size: string | null
  unit: string
  hsn_code: string | null
  gst_rate: number
  barcode: string | null
  is_prescription_required: boolean
  is_active: boolean
  schedule_class: MedicineSchedule
  is_narcotic: boolean
  units_per_pack: number
}

export interface ProductInput {
  sku: string
  name: string
  generic_name?: string | null
  brand_name?: string | null
  brand_id?: string | null
  composition_ids?: string[]
  dosage_form?: string | null
  strength?: string | null
  pack_size?: string | null
  unit?: string
  hsn_code?: string | null
  gst_rate?: number
  barcode?: string | null
  is_prescription_required?: boolean
  is_active?: boolean
  schedule_class?: MedicineSchedule
  is_narcotic?: boolean
  units_per_pack?: number
}

export type ProductList = ListResponse<Product>

// ─── Brands ──────────────────────────────────────────────────

export interface Brand {
  id: string
  name: string
  manufacturer_id: string | null
  description: string | null
  is_active: boolean
}

export interface BrandInput {
  name: string
  manufacturer_id?: string | null
  description?: string | null
  is_active?: boolean
}

// ─── Compositions ────────────────────────────────────────────

export interface Composition {
  id: string
  name: string
  description: string | null
  is_active: boolean
}

export interface CompositionInput {
  name: string
  description?: string | null
  is_active?: boolean
}

// ─── HSN Master ──────────────────────────────────────────────

export interface Hsn {
  hsn_code: string
  description: string
  gst_rate: number
  cgst_rate: number
  sgst_rate: number
  igst_rate: number
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface HsnInput {
  hsn_code: string
  description?: string
  gst_rate: number
  cgst_rate: number
  sgst_rate: number
  igst_rate: number
}

export type HsnList = ListResponse<Hsn>

export interface HsnQuery {
  search?: string
  page?: number
  limit?: number
  active?: boolean
}

export interface HsnImportError {
  row: number
  hsn_code: string
  error: string
}

export interface HsnImportResult {
  total_rows: number
  inserted: number
  skipped: number
  rejected: number
  errors: HsnImportError[]
}

// ─── Inventory ───────────────────────────────────────────────

export interface InventorySummary {
  total_products: number
  total_batches: number
  total_units: number
  low_stock: number
  out_of_stock: number
  expiring_soon: number
  expired: number
  stock_value: number
}

export interface InventoryBatch {
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

export type InventoryBatchList = ListResponse<InventoryBatch>

export interface StockMovement {
  id: string
  product_id: string
  batch_id: string
  movement_type: string
  quantity_change: number
  quantity_before: number
  quantity_after: number
  reason: string
  created_at: string
}

// ─── Parties ─────────────────────────────────────────────────

export type PartyRole = 'SUPPLIER' | 'CUSTOMER'

export type OpeningBalanceType = 'PAYABLE' | 'RECEIVABLE' | 'ADVANCE'

export interface Party {
  id: string
  party_code: string
  name: string
  contact_person: string | null
  phone: string | null
  alternate_phone: string | null
  email: string | null
  gstin: string | null
  pan: string | null
  drug_license_number: string | null
  billing_address: string | null
  shipping_address: string | null
  city: string | null
  district: string | null
  state: string
  pincode: string
  country: string
  opening_balance: number
  opening_balance_type: OpeningBalanceType
  credit_limit: number
  credit_days: number
  notes: string | null
  is_active: boolean
  created_at: string
  updated_at: string
  roles: PartyRole[]
}

export interface PartyInput {
  party_code: string
  name: string
  roles: PartyRole[]

  contact_person?: string | null
  phone?: string | null
  alternate_phone?: string | null
  email?: string | null

  gstin?: string | null
  pan?: string | null
  drug_license_number?: string | null

  billing_address?: string | null
  shipping_address?: string | null
  city?: string | null
  district?: string | null
  state?: string
  pincode?: string
  country?: string

  opening_balance?: number
  opening_balance_type?: OpeningBalanceType
  credit_limit?: number
  credit_days?: number

  notes?: string | null
  is_active?: boolean
}

export type PartyList = ListResponse<Party>

// ─── Purchases ─────────────────────────────────────

export type PurchaseStatus = 'DRAFT' | 'POSTED' | 'CANCELLED'
export type PurchaseTaxMode = 'INCLUSIVE' | 'EXCLUSIVE'
export type PurchaseTaxType = 'INTRA_STATE' | 'INTER_STATE'

export interface PurchaseListItem {
  id: string
  purchase_number: string
  supplier_id: string
  supplier_name: string
  supplier_invoice_number: string
  invoice_date: string
  due_date: string | null
  status: PurchaseStatus
  grand_total: number
  paid_amount: number
  created_at: string
}

export interface PurchaseItemInput {
  product_id?: string
  new_product?: {
    sku?: string | null
    name: string
    generic_name?: string | null
    brand_id?: string | null
    brand_name?: string | null
    composition_ids?: string[]
    dosage_form?: string | null
    strength?: string | null
    pack_size?: string | null
    unit?: string
    hsn_code?: string | null
    barcode?: string | null
    is_prescription_required?: boolean
    schedule_class?: MedicineSchedule
    is_narcotic?: boolean
    units_per_pack?: number
  }
  product_updates?: {
    brand_id?: string | null
    brand_name?: string | null
    dosage_form?: string | null
    hsn_code?: string | null
    schedule_class?: MedicineSchedule
    is_narcotic?: boolean
    is_prescription_required?: boolean
  }
  batch_number: string
  manufacturing_date?: string | null
  expiry_date?: string | null
  quantity: number
  free_quantity?: number
  purchase_rate: number
  mrp: number
  selling_rate?: number
  discount_percent?: number
}

export interface PurchaseInput {
  supplier_id: string
  supplier_invoice_number: string
  invoice_date: string
  due_date?: string | null
  notes?: string | null
  tax_mode: PurchaseTaxMode
  tax_type: PurchaseTaxType
  items: PurchaseItemInput[]
}

export interface PurchaseDetailItem {
  id: string
  line_number: number
  product_id: string | null
  product_name: string
  sku: string
  product_payload: string | null
  batch_id: string | null
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

export interface PurchaseDetail {
  purchase: PurchaseListItem & {
    tax_mode: PurchaseTaxMode
    tax_type: PurchaseTaxType
    subtotal: number
    discount_amount: number
    taxable_amount: number
    tax_amount: number
    other_charges: number
    round_off: number
    notes: string | null
  }
  items: PurchaseDetailItem[]
}

// ─── API Client ──────────────────────────────────────────────

export function useApi() {
  // ─── Products ─────────────────────────────────────────────

  async function getProducts(
    params: {
      search?: string
      page?: number
      limit?: number
    } = {},
  ): Promise<ProductList> {
    return $fetch<ProductList>(`${API_BASE_URL}/products`, {
      query: params,
    })
  }

  async function getProduct(id: string): Promise<Product> {
    return $fetch<Product>(
      `${API_BASE_URL}/products/${encodeURIComponent(id)}`,
    )
  }

  async function createProduct(payload: ProductInput): Promise<Product> {
    return $fetch<Product>(`${API_BASE_URL}/products`, {
      method: 'POST',
      body: payload,
    })
  }

  async function updateProduct(
    id: string,
    payload: ProductInput,
  ): Promise<Product> {
    return $fetch<Product>(
      `${API_BASE_URL}/products/${encodeURIComponent(id)}`,
      {
        method: 'PUT',
        body: payload,
      },
    )
  }

  async function deleteProduct(id: string): Promise<void> {
    return $fetch<void>(
      `${API_BASE_URL}/products/${encodeURIComponent(id)}`,
      {
        method: 'DELETE',
      },
    )
  }

  // ─── Brands ───────────────────────────────────────────────

  async function getBrands(
    params: {
      search?: string
      page?: number
      limit?: number
    } = {},
  ): Promise<ListResponse<Brand>> {
    return $fetch<ListResponse<Brand>>(`${API_BASE_URL}/brands`, {
      query: params,
    })
  }

  async function createBrand(payload: BrandInput): Promise<Brand> {
    return $fetch<Brand>(`${API_BASE_URL}/brands`, {
      method: 'POST',
      body: payload,
    })
  }

  async function updateBrand(
    id: string,
    payload: BrandInput,
  ): Promise<Brand> {
    return $fetch<Brand>(
      `${API_BASE_URL}/brands/${encodeURIComponent(id)}`,
      {
        method: 'PUT',
        body: payload,
      },
    )
  }

  async function deleteBrand(id: string): Promise<void> {
    return $fetch<void>(
      `${API_BASE_URL}/brands/${encodeURIComponent(id)}`,
      {
        method: 'DELETE',
      },
    )
  }

  // ─── Compositions ─────────────────────────────────────────

  async function getCompositions(
    params: {
      search?: string
      page?: number
      limit?: number
    } = {},
  ): Promise<ListResponse<Composition>> {
    return $fetch<ListResponse<Composition>>(
      `${API_BASE_URL}/compositions`,
      {
        query: params,
      },
    )
  }

  async function createComposition(
    payload: CompositionInput,
  ): Promise<Composition> {
    return $fetch<Composition>(`${API_BASE_URL}/compositions`, {
      method: 'POST',
      body: payload,
    })
  }

  async function updateComposition(
    id: string,
    payload: CompositionInput,
  ): Promise<Composition> {
    return $fetch<Composition>(
      `${API_BASE_URL}/compositions/${encodeURIComponent(id)}`,
      {
        method: 'PUT',
        body: payload,
      },
    )
  }

  async function deleteComposition(id: string): Promise<void> {
    return $fetch<void>(
      `${API_BASE_URL}/compositions/${encodeURIComponent(id)}`,
      {
        method: 'DELETE',
      },
    )
  }

  // ─── HSN Master ───────────────────────────────────────────

  async function getHsn(params: HsnQuery = {}): Promise<HsnList> {
    return $fetch<HsnList>(`${API_BASE_URL}/hsn`, {
      query: params,
    })
  }

  async function getHsnByCode(code: string): Promise<Hsn> {
    return $fetch<Hsn>(
      `${API_BASE_URL}/hsn/${encodeURIComponent(code)}`,
    )
  }

  async function createHsn(payload: HsnInput): Promise<Hsn> {
    return $fetch<Hsn>(`${API_BASE_URL}/hsn`, {
      method: 'POST',
      body: payload,
    })
  }

  async function updateHsn(
    code: string,
    payload: HsnInput,
  ): Promise<Hsn> {
    return $fetch<Hsn>(
      `${API_BASE_URL}/hsn/${encodeURIComponent(code)}`,
      {
        method: 'PUT',
        body: payload,
      },
    )
  }

  async function updateHsnStatus(
    code: string,
    is_active: boolean,
  ): Promise<Hsn> {
    return $fetch<Hsn>(
      `${API_BASE_URL}/hsn/${encodeURIComponent(code)}/status`,
      {
        method: 'PATCH',
        body: { is_active },
      },
    )
  }

  async function importHsnCsv(file: File): Promise<HsnImportResult> {
    const formData = new FormData()
    formData.append('file', file)

    return $fetch<HsnImportResult>(`${API_BASE_URL}/hsn/import`, {
      method: 'POST',
      body: formData,
    })
  }

  // ─── Inventory ────────────────────────────────────────────

  async function getInventorySummary(): Promise<InventorySummary> {
    return $fetch<InventorySummary>(
      `${API_BASE_URL}/inventory/summary`,
    )
  }

  async function getInventoryBatches(
    params: {
      search?: string
      page?: number
      limit?: number
      stock_status?: string
      expiry_status?: string
    } = {},
  ): Promise<InventoryBatchList> {
    return $fetch<InventoryBatchList>(
      `${API_BASE_URL}/inventory/batches`,
      {
        query: params,
      },
    )
  }

  async function getStockAdjustments(
    params: {
      page?: number
      limit?: number
      product_id?: string
      batch_id?: string
    } = {},
  ): Promise<ListResponse<StockMovement>> {
    return $fetch<ListResponse<StockMovement>>(
      `${API_BASE_URL}/inventory/adjustments`,
      {
        query: params,
      },
    )
  }

  async function createStockAdjustment(payload: {
    product_id: string
    batch_id: string
    movement_type: 'IN' | 'OUT' | 'ADJUSTMENT'
    quantity: number
    reason: string
  }): Promise<StockMovement> {
    return $fetch<StockMovement>(
      `${API_BASE_URL}/inventory/adjustments`,
      {
        method: 'POST',
        body: payload,
      },
    )
  }

  // ─── Parties ─────────────────────────────────────────────

  async function getParties(
    params: {
      search?: string
      role?: PartyRole
      active?: boolean
      page?: number
      limit?: number
    } = {},
  ): Promise<PartyList> {
    return $fetch<PartyList>(`${API_BASE_URL}/parties`, {
      query: params,
    })
  }

  async function getParty(id: string): Promise<Party> {
    return $fetch<Party>(
      `${API_BASE_URL}/parties/${encodeURIComponent(id)}`,
    )
  }

  async function createParty(payload: PartyInput): Promise<Party> {
    return $fetch<Party>(`${API_BASE_URL}/parties`, {
      method: 'POST',
      body: payload,
    })
  }

  async function updateParty(
    id: string,
    payload: PartyInput,
  ): Promise<Party> {
    return $fetch<Party>(
      `${API_BASE_URL}/parties/${encodeURIComponent(id)}`,
      {
        method: 'PUT',
        body: payload,
      },
    )
  }

  async function deactivateParty(
    id: string,
  ): Promise<{ message: string }> {
    return $fetch<{ message: string }>(
      `${API_BASE_URL}/parties/${encodeURIComponent(id)}`,
      {
        method: 'DELETE',
      },
    )
  }
  // ─── Purchases ─────────────────────────────────────

  async function getPurchases(
    params: {
      search?: string
      status?: PurchaseStatus | 'ALL'
      page?: number
      limit?: number
    } = {},
  ): Promise<ListResponse<PurchaseListItem>> {
    return $fetch<ListResponse<PurchaseListItem>>(
      `${API_BASE_URL}/purchases`,
      { query: params },
    )
  }

  async function getPurchase(id: string): Promise<PurchaseDetail> {
    return $fetch<PurchaseDetail>(
      `${API_BASE_URL}/purchases/${encodeURIComponent(id)}`,
    )
  }

  async function createPurchase(
    payload: PurchaseInput,
  ): Promise<PurchaseDetail> {
    return $fetch<PurchaseDetail>(
      `${API_BASE_URL}/purchases`,
      {
        method: 'POST',
        body: payload,
      },
    )
  }

  async function updatePurchase(
    id: string,
    payload: PurchaseInput,
  ): Promise<PurchaseDetail> {
    return $fetch<PurchaseDetail>(
      `${API_BASE_URL}/purchases/${encodeURIComponent(id)}`,
      {
        method: 'PUT',
        body: payload,
      },
    )
  }

  async function deletePurchase(id: string): Promise<void> {
    return $fetch<void>(
      `${API_BASE_URL}/purchases/${encodeURIComponent(id)}`,
      { method: 'DELETE' },
    )
  }

  async function postPurchase(id: string): Promise<PurchaseDetail> {
    return $fetch<PurchaseDetail>(
      `${API_BASE_URL}/purchases/${encodeURIComponent(id)}`,
      { method: 'POST' },
    )
  }

  // ─── Public API ───────────────────────────────────────────

  return {
    // Products
    getProducts,
    getProduct,
    createProduct,
    updateProduct,
    deleteProduct,

    // Brands
    getBrands,
    createBrand,
    updateBrand,
    deleteBrand,

    // Compositions
    getCompositions,
    createComposition,
    updateComposition,
    deleteComposition,

    // HSN Master
    getHsn,
    getHsnByCode,
    createHsn,
    updateHsn,
    updateHsnStatus,
    importHsnCsv,

    // Inventory
    getInventorySummary,
    getInventoryBatches,
    getStockAdjustments,
    createStockAdjustment,

    // Parties
    getParties,
    getParty,
    createParty,
    updateParty,
    deactivateParty,

    // Purchases
    getPurchases,
    getPurchase,
    createPurchase,
    updatePurchase,
    deletePurchase,
    postPurchase,
  }
}
