<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import {
  X,
  Plus,
  Check,
  ChevronDown,
  LoaderCircle,
  Search,
  Package,
  Pill,
  ShieldCheck,
  ReceiptText,
  Barcode,
} from 'lucide-vue-next'

import { useApi } from '~/composables/useApi'

import type {
  Brand,
  Composition,
  Hsn,
  HsnInput,
  ProductInput,
} from '~/composables/useApi'

type MasterType = 'brand' | 'composition' | 'hsn'
type Schedule = 'OTC' | 'H' | 'H1' | 'X'

const api = useApi()

const props = defineProps<{
  modelValue: boolean
  mode: 'create' | 'edit'
  form: ProductInput
  brands: Brand[]
  compositions: Composition[]
  brandSearch: string
  compositionSearch: string
  newBrandName: string
  newCompositionName: string
  addingBrand: boolean
  addingComposition: boolean
  masterLoading: boolean
  saving: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'save'): void
  (e: 'add-brand'): void
  (e: 'add-composition'): void
  (e: 'update:brandSearch', value: string): void
  (e: 'update:compositionSearch', value: string): void
  (e: 'update:newBrandName', value: string): void
  (e: 'update:newCompositionName', value: string): void
}>()

const activeDropdown = ref<MasterType | null>(null)
const activePopup = ref<MasterType | null>(null)

const popupName = ref('')
const popupError = ref('')

const dropdownRoot = ref<HTMLElement | null>(null)

const hsnSearch = ref('')
const hsnRecords = ref<Hsn[]>([])
const hsnLoading = ref(false)
const hsnSaving = ref(false)
const hsnError = ref('')
const hsnLoaded = ref(false)

const newHsnCode = ref('')
const newHsnDescription = ref('')
const newHsnGstRate = ref<number>(0)

const brandQuery = ref('')
const compositionQuery = ref('')

const scheduleOptions: {
  value: Schedule
  label: string
  description: string
  color: string
}[] = [
  {
    value: 'OTC',
    label: 'OTC',
    description: 'Over the counter',
    color: 'green',
  },
  {
    value: 'H',
    label: 'Schedule H',
    description: 'Prescription medicine',
    color: 'amber',
  },
  {
    value: 'H1',
    label: 'Schedule H1',
    description: 'Restricted prescription',
    color: 'orange',
  },
  {
    value: 'X',
    label: 'Schedule X',
    description: 'Special regulatory controls',
    color: 'rose',
  },
]

const dosageForms = [
  'Tablet',
  'Capsule',
  'Syrup',
  'Suspension',
  'Injection',
  'Cream',
  'Ointment',
  'Gel',
  'Drops',
  'Inhaler',
  'Powder',
  'Solution',
  'Other',
]

const units = [
  'PCS',
  'STRIP',
  'BOX',
  'BOTTLE',
  'VIAL',
  'TUBE',
  'PACK',
  'KG',
  'GM',
  'ML',
  'LITRE',
]

const filteredBrands = computed(() => {
  const query = props.brandSearch.trim().toLowerCase()

  return props.brands.filter(brand =>
    brand.name.toLowerCase().includes(query),
  )
})

const filteredCompositions = computed(() => {
  const query = props.compositionSearch.trim().toLowerCase()

  return props.compositions.filter(composition =>
    composition.name.toLowerCase().includes(query),
  )
})

const filteredHsn = computed(() => {
  const query = hsnSearch.value.trim().toLowerCase()

  return hsnRecords.value.filter(item => {
    if (!item.is_active) return false

    return (
      item.hsn_code.toLowerCase().includes(query) ||
      item.description.toLowerCase().includes(query)
    )
  })
})

const selectedHsn = computed(() => {
  const code = String(props.form.hsn_code || '').trim()

  if (!code) return undefined

  return hsnRecords.value.find(item => item.hsn_code === code)
})

const selectedBrand = computed(() =>
  props.brands.find(brand => brand.id === props.form.brand_id),
)

const selectedComposition = computed(() => {
  const id = props.form.composition_ids?.[0]

  return props.compositions.find(item => item.id === id)
})

const packSummary = computed(() => {
  const quantity = Number(props.form.units_per_pack || 1)
  const unit = props.form.unit || 'PCS'
  const pack = props.form.pack_size?.trim()

  return `${pack || '1 pack'} · ${quantity} ${unit}`
})

const hsnGstBreakdown = computed(() => {
  const rate = Number(props.form.gst_rate || 0)

  return {
    cgst: rate / 2,
    sgst: rate / 2,
  }
})

function toggleDropdown(type: MasterType) {
  activeDropdown.value =
    activeDropdown.value === type ? null : type

  if (type === 'brand') {
    emit('update:brandSearch', '')
  }

  if (type === 'composition') {
    emit('update:compositionSearch', '')
  }

  if (type === 'hsn' && activeDropdown.value === 'hsn') {
    hsnSearch.value = ''
    void loadHsn()
  }
}

function selectBrand(brand: Brand) {
  props.form.brand_id = brand.id
  props.form.brand_name = brand.name

  emit('update:brandSearch', '')
  activeDropdown.value = null
}

function clearBrand() {
  props.form.brand_id = null
  props.form.brand_name = ''

  emit('update:brandSearch', '')
  activeDropdown.value = null
}

function selectComposition(composition: Composition) {
  props.form.composition_ids = [composition.id]

  emit('update:compositionSearch', '')
  activeDropdown.value = null
}

function clearComposition() {
  props.form.composition_ids = []

  activeDropdown.value = null
  emit('update:compositionSearch', '')
}

/**
 * HSN Master
 *
 * Existing HSN records are loaded from the backend.
 * Only active HSN codes are offered for selection.
 */
async function loadHsn(force = false) {
  if (hsnLoading.value) return
  if (hsnLoaded.value && !force) return

  hsnLoading.value = true
  hsnError.value = ''

  try {
    const response = await api.getHsn({
      active: true,
      page: 1,
      limit: 500,
    })

    hsnRecords.value = response.data
    hsnLoaded.value = true
  } catch (error: unknown) {
    hsnError.value = getErrorMessage(
      error,
      'Unable to load HSN master records.',
    )
  } finally {
    hsnLoading.value = false
  }
}

function selectHsn(item: Hsn) {
  props.form.hsn_code = item.hsn_code
  props.form.gst_rate = Number(item.gst_rate)

  hsnSearch.value = ''
  hsnError.value = ''
  activeDropdown.value = null
}

function clearHsn() {
  props.form.hsn_code = ''
  props.form.gst_rate = 0

  hsnSearch.value = ''
  activeDropdown.value = null
}

function openAddPopup(type: MasterType) {
  activeDropdown.value = null
  activePopup.value = type

  popupName.value = ''
  popupError.value = ''

  if (type === 'hsn') {
    newHsnCode.value = hsnSearch.value.trim()
    newHsnDescription.value = ''
    newHsnGstRate.value = Number(props.form.gst_rate || 0)
    hsnError.value = ''
  }
}

function closeAddPopup() {
  activePopup.value = null

  popupName.value = ''
  popupError.value = ''
}

function submitNewMaster() {
  const name = popupName.value.trim()

  if (!name) {
    popupError.value = 'Please enter a name.'
    return
  }

  if (activePopup.value === 'brand') {
    emit('update:newBrandName', name)
    emit('add-brand')
  }

  if (activePopup.value === 'composition') {
    emit('update:newCompositionName', name)
    emit('add-composition')
  }

  closeAddPopup()
}

function getErrorMessage(error: unknown, fallback: string): string {
  if (
    error &&
    typeof error === 'object' &&
    'data' in error
  ) {
    const data = (error as { data?: { message?: string; error?: string } }).data

    if (data?.message) return data.message
    if (data?.error) return data.error
  }

  if (error instanceof Error && error.message) {
    return error.message
  }

  return fallback
}

/**
 * Creates an HSN record through the existing HSN API.
 *
 * The GST split is calculated as CGST + SGST for the
 * entered total GST rate. This assumes an intra-state
 * transaction; the actual invoice tax treatment should
 * be handled by the billing/tax logic.
 */
async function submitNewHsn() {
  const code = newHsnCode.value.trim()
  const description = newHsnDescription.value.trim()
  const gstRate = Number(newHsnGstRate.value)

  hsnError.value = ''

  if (!code) {
    hsnError.value = 'Please enter an HSN code.'
    return
  }

  if (!/^\d{4,8}$/.test(code)) {
    hsnError.value = 'Enter a valid 4–8 digit HSN code.'
    return
  }

  if (!Number.isFinite(gstRate) || gstRate < 0 || gstRate > 100) {
    hsnError.value = 'GST rate must be between 0 and 100.'
    return
  }

  const duplicate = hsnRecords.value.find(
    item => item.hsn_code === code,
  )

  if (duplicate) {
    hsnError.value = 'This HSN code already exists in HSN Master.'

    if (duplicate.is_active) {
      selectHsn(duplicate)
      closeAddPopup()
    }

    return
  }

  const payload: HsnInput = {
    hsn_code: code,
    description,
    gst_rate: gstRate,
    cgst_rate: gstRate / 2,
    sgst_rate: gstRate / 2,
    igst_rate: gstRate,
  }

  hsnSaving.value = true

  try {
    const created = await api.createHsn(payload)

    await loadHsn(true)

    const record =
      hsnRecords.value.find(
        item => item.hsn_code === created.hsn_code,
      ) || created

    selectHsn(record)
    closeAddPopup()
  } catch (error: unknown) {
    hsnError.value = getErrorMessage(
      error,
      'Unable to create HSN record.',
    )
  } finally {
    hsnSaving.value = false
  }
}

function onDocumentClick(event: MouseEvent) {
  if (
    dropdownRoot.value &&
    !dropdownRoot.value.contains(event.target as Node)
  ) {
    activeDropdown.value = null
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    if (activePopup.value) {
      closeAddPopup()
    } else if (activeDropdown.value) {
      activeDropdown.value = null
    } else {
      emit('close')
    }
  }
}

onMounted(() => {
  document.addEventListener('click', onDocumentClick)
  document.addEventListener('keydown', onKeydown)

  void loadHsn()
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocumentClick)
  document.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="pfm-fade">
      <div
        v-if="modelValue"
        class="pfm-backdrop"
        @click.self="emit('close')"
      >
        <section
          ref="dropdownRoot"
          class="pfm-modal"
          role="dialog"
          aria-modal="true"
          :aria-label="mode === 'create' ? 'Add product' : 'Edit product'"
        >
          <header class="pfm-header">
            <div class="pfm-heading-icon">
              <Package :size="21" />
            </div>

            <div class="pfm-heading-copy">
              <span class="pfm-eyebrow">
                {{ mode === 'create' ? 'PRODUCT CATALOGUE' : 'UPDATE RECORD' }}
              </span>

              <h2>
                {{ mode === 'create' ? 'Add product' : 'Edit product' }}
              </h2>

              <p>Manage product information, packaging and compliance.</p>
            </div>

            <button
              type="button"
              class="pfm-close"
              aria-label="Close form"
              @click="emit('close')"
            >
              <X :size="18" />
            </button>
          </header>

          <form class="pfm-form" @submit.prevent="emit('save')">
            <!-- Product information -->
            <section class="pfm-section">
              <div class="pfm-section-heading">
                <div class="pfm-section-icon green">
                  <Package :size="16" />
                </div>

                <div>
                  <h3>Product information</h3>
                  <p>Basic identity and medicine details</p>
                </div>
              </div>

              <div class="pfm-grid">
                <label class="pfm-field">
                  <span>Product name <b>*</b></span>

                  <input
                    v-model="form.name"
                    required
                    maxlength="250"
                    placeholder="e.g. Paracetamol Tablets"
                  />
                </label>

                <label class="pfm-field">
                  <span>SKU <b>*</b></span>

                  <input
                    v-model="form.sku"
                    required
                    maxlength="100"
                    placeholder="e.g. MED-00001"
                  />
                </label>

                <!-- Brand -->
                <div class="pfm-field pfm-full">
                  <span>Brand</span>

                  <div class="pfm-master-row">
                    <div class="pfm-dropdown">
                      <button
                        type="button"
                        class="pfm-select-trigger"
                        :class="{ 'is-open': activeDropdown === 'brand' }"
                        @click.stop="toggleDropdown('brand')"
                      >
                        <span
                          class="pfm-trigger-value"
                          :class="{ muted: !selectedBrand }"
                        >
                          {{ selectedBrand?.name || 'Select a brand' }}
                        </span>

                        <ChevronDown
                          :size="16"
                          class="pfm-chevron"
                          :class="{ rotated: activeDropdown === 'brand' }"
                        />
                      </button>

                      <Transition name="pfm-menu">
                        <div
                          v-if="activeDropdown === 'brand'"
                          class="pfm-dropdown-menu"
                          @click.stop
                        >
                          <div class="pfm-menu-search">
                            <Search :size="15" />

                            <input
                              :value="brandSearch"
                              placeholder="Search brands..."
                              autocomplete="off"
                              @input="emit('update:brandSearch', ($event.target as HTMLInputElement).value)"
                            />
                          </div>

                          <div class="pfm-menu-list">
                            <button
                              type="button"
                              class="pfm-option"
                              @click="clearBrand"
                            >
                              No brand
                            </button>

                            <button
                              v-for="brand in filteredBrands"
                              :key="brand.id"
                              type="button"
                              class="pfm-option"
                              @click="selectBrand(brand)"
                            >
                              <span>{{ brand.name }}</span>

                              <Check
                                v-if="form.brand_id === brand.id"
                                :size="16"
                              />
                            </button>

                            <div
                              v-if="!filteredBrands.length"
                              class="pfm-empty"
                            >
                              No brands found
                            </div>
                          </div>

                          <div class="pfm-menu-footer">
                            <span>{{ filteredBrands.length }} brands</span>

                            <button
                              type="button"
                              @click="openAddPopup('brand')"
                            >
                              <Plus :size="14" />
                              Add new brand
                            </button>
                          </div>
                        </div>
                      </Transition>
                    </div>

                    <button
                      type="button"
                      class="pfm-add-button"
                      title="Add new brand"
                      @click="openAddPopup('brand')"
                    >
                      <Plus :size="17" />
                    </button>
                  </div>
                </div>

                <!-- Composition -->
                <div class="pfm-field pfm-full">
                  <span>Composition / active ingredient</span>

                  <div class="pfm-master-row">
                    <div class="pfm-dropdown">
                      <button
                        type="button"
                        class="pfm-select-trigger"
                        :class="{ 'is-open': activeDropdown === 'composition' }"
                        @click.stop="toggleDropdown('composition')"
                      >
                        <span
                          class="pfm-trigger-value"
                          :class="{ muted: !selectedComposition }"
                        >
                          {{ selectedComposition?.name || 'Select a composition' }}
                        </span>

                        <ChevronDown
                          :size="16"
                          class="pfm-chevron"
                          :class="{ rotated: activeDropdown === 'composition' }"
                        />
                      </button>

                      <Transition name="pfm-menu">
                        <div
                          v-if="activeDropdown === 'composition'"
                          class="pfm-dropdown-menu"
                          @click.stop
                        >
                          <div class="pfm-menu-search">
                            <Search :size="15" />

                            <input
                              :value="compositionSearch"
                              placeholder="Search compositions..."
                              autocomplete="off"
                              @input="emit('update:compositionSearch', ($event.target as HTMLInputElement).value)"
                            />
                          </div>

                          <div class="pfm-menu-list">
                            <button
                              type="button"
                              class="pfm-option"
                              @click="clearComposition"
                            >
                              No composition
                            </button>

                            <button
                              v-for="composition in filteredCompositions"
                              :key="composition.id"
                              type="button"
                              class="pfm-option"
                              @click="selectComposition(composition)"
                            >
                              <span>{{ composition.name }}</span>

                              <Check
                                v-if="form.composition_ids?.[0] === composition.id"
                                :size="16"
                              />
                            </button>

                            <div
                              v-if="!filteredCompositions.length"
                              class="pfm-empty"
                            >
                              No compositions found
                            </div>
                          </div>

                          <div class="pfm-menu-footer">
                            <span>{{ filteredCompositions.length }} compositions</span>

                            <button
                              type="button"
                              @click="openAddPopup('composition')"
                            >
                              <Plus :size="14" />
                              Add composition
                            </button>
                          </div>
                        </div>
                      </Transition>
                    </div>

                    <button
                      type="button"
                      class="pfm-add-button"
                      title="Add composition"
                      @click="openAddPopup('composition')"
                    >
                      <Plus :size="17" />
                    </button>
                  </div>

                  <span v-if="masterLoading" class="pfm-help">
                    <LoaderCircle :size="13" class="pfm-spin" />
                    Loading master records
                  </span>
                </div>

                <label class="pfm-field">
                  <span>Dosage form</span>

                  <select v-model="form.dosage_form">
                    <option value="">Select form</option>

                    <option
                      v-for="dosage in dosageForms"
                      :key="dosage"
                      :value="dosage"
                    >
                      {{ dosage }}
                    </option>
                  </select>
                </label>

                <label class="pfm-field">
                  <span>Strength</span>

                  <input
                    v-model="form.strength"
                    placeholder="e.g. 500 mg"
                  />
                </label>

                <label class="pfm-field">
                  <span>Pack size</span>

                  <input
                    v-model="form.pack_size"
                    placeholder="e.g. 10 tablets"
                  />
                </label>

                <label class="pfm-field">
                  <span>Units per pack <b>*</b></span>

                  <input
                    v-model.number="form.units_per_pack"
                    type="number"
                    min="1"
                    step="1"
                    required
                  />

                  <span class="pfm-help">{{ packSummary }}</span>
                </label>

                <label class="pfm-field">
                  <span>Stock unit</span>

                  <select v-model="form.unit">
                    <option
                      v-for="unit in units"
                      :key="unit"
                      :value="unit"
                    >
                      {{ unit }}
                    </option>
                  </select>
                </label>
              </div>
            </section>

            <!-- Tax & identification -->
            <section class="pfm-section">
              <div class="pfm-section-heading">
                <div class="pfm-section-icon purple">
                  <ReceiptText :size="16" />
                </div>

                <div>
                  <h3>Tax & identification</h3>
                  <p>Tax classification and product identification</p>
                </div>
              </div>

              <div class="pfm-grid">
                <!-- Searchable HSN dropdown -->
                <div class="pfm-field">
                  <span>HSN code</span>

                  <div class="pfm-dropdown">
                    <button
                      type="button"
                      class="pfm-select-trigger"
                      :class="{ 'is-open': activeDropdown === 'hsn' }"
                      @click.stop="toggleDropdown('hsn')"
                    >
                      <span
                        class="pfm-trigger-value"
                        :class="{ muted: !form.hsn_code }"
                      >
                        {{ form.hsn_code || 'Search HSN code or description' }}
                      </span>

                      <ChevronDown
                        :size="16"
                        class="pfm-chevron"
                        :class="{ rotated: activeDropdown === 'hsn' }"
                      />
                    </button>

                    <Transition name="pfm-menu">
                      <div
                        v-if="activeDropdown === 'hsn'"
                        class="pfm-dropdown-menu"
                        @click.stop
                      >
                        <div class="pfm-menu-search">
                          <Search :size="15" />

                          <input
                            v-model="hsnSearch"
                            autofocus
                            autocomplete="off"
                            placeholder="Search code or description..."
                          />

                          <LoaderCircle
                            v-if="hsnLoading"
                            :size="15"
                            class="pfm-spin"
                          />
                        </div>

                        <div class="pfm-menu-list">
                          <button
                            type="button"
                            class="pfm-option"
                            @click="clearHsn"
                          >
                            <span class="pfm-option-text">
                              <span class="pfm-option-name">No HSN</span>
                              <span class="pfm-option-sub">Clear HSN selection</span>
                            </span>
                          </button>

                          <button
                            v-for="item in filteredHsn"
                            :key="item.hsn_code"
                            type="button"
                            class="pfm-option"
                            :class="{
                              'is-selected': form.hsn_code === item.hsn_code,
                            }"
                            @click="selectHsn(item)"
                          >
                            <span class="pfm-option-text">
                              <span class="pfm-option-name">
                                {{ item.hsn_code }}
                              </span>

                              <span class="pfm-option-sub">
                                {{ item.description || 'No description' }}
                              </span>
                            </span>

                            <span class="pfm-hsn-rate">
                              {{ Number(item.gst_rate) }}% GST
                            </span>

                            <Check
                              v-if="form.hsn_code === item.hsn_code"
                              :size="16"
                              class="pfm-option-check"
                            />
                          </button>

                          <div
                            v-if="!hsnLoading && !filteredHsn.length"
                            class="pfm-empty"
                          >
                            <Search :size="20" />

                            <span>
                              {{ hsnSearch ? 'No matching HSN found' : 'No active HSN records' }}
                            </span>
                          </div>

                          <div
                            v-if="hsnError"
                            class="pfm-inline-error"
                          >
                            {{ hsnError }}
                          </div>
                        </div>

                        <div class="pfm-menu-footer">
                          <span>
                            {{ filteredHsn.length }} HSN records
                          </span>

                          <button
                            type="button"
                            @click="openAddPopup('hsn')"
                          >
                            <Plus :size="14" />
                            Add New HSN
                          </button>
                        </div>
                      </div>
                    </Transition>
                  </div>

                  <span v-if="selectedHsn" class="pfm-help">
                    {{ selectedHsn.description || 'HSN selected' }}
                  </span>
                </div>

                <!-- GST rate -->
                <label class="pfm-field">
                  <span>GST rate (%)</span>

                  <div class="pfm-input-suffix">
                    <input
                      v-model.number="form.gst_rate"
                      type="number"
                      min="0"
                      max="100"
                      step="0.01"
                    />

                    <span>%</span>
                  </div>

                  <span v-if="selectedHsn" class="pfm-help">
                    CGST {{ hsnGstBreakdown.cgst }}% ·
                    SGST {{ hsnGstBreakdown.sgst }}%
                  </span>
                </label>

                <label class="pfm-field pfm-full">
                  <span>Barcode</span>

                  <div class="pfm-input-prefix">
                    <Barcode :size="16" />

                    <input
                      v-model="form.barcode"
                      placeholder="Scan or enter barcode"
                    />
                  </div>
                </label>
              </div>
            </section>

            <!-- Regulatory classification -->
            <section class="pfm-section">
              <div class="pfm-section-heading">
                <div class="pfm-section-icon amber">
                  <ShieldCheck :size="16" />
                </div>

                <div>
                  <h3>Regulatory classification</h3>
                  <p>Select the applicable medicine schedule</p>
                </div>
              </div>

              <div class="pfm-schedules">
                <label
                  v-for="option in scheduleOptions"
                  :key="option.value"
                  class="pfm-schedule"
                  :class="[
                    `schedule-${option.color}`,
                    { 'is-selected': form.schedule_class === option.value },
                  ]"
                >
                  <input
                    v-model="form.schedule_class"
                    type="radio"
                    name="product-schedule"
                    :value="option.value"
                  />

                  <span class="pfm-schedule-check">
                    <Check :size="13" />
                  </span>

                  <span class="pfm-schedule-copy">
                    <strong>{{ option.label }}</strong>
                    <small>{{ option.description }}</small>
                  </span>
                </label>
              </div>

              <div class="pfm-settings">
                <label class="pfm-setting">
                  <span class="pfm-checkbox">
                    <input
                      v-model="form.is_prescription_required"
                      type="checkbox"
                    />

                    <Check :size="13" />
                  </span>

                  <span class="pfm-setting-copy">
                    <strong>Prescription required</strong>
                    <small>Prescription requirements apply to this product.</small>
                  </span>
                </label>

                <label class="pfm-setting">
                  <span class="pfm-checkbox">
                    <input
                      v-model="form.is_narcotic"
                      type="checkbox"
                    />

                    <Check :size="13" />
                  </span>

                  <span class="pfm-setting-copy">
                    <strong>Narcotic / controlled medicine</strong>
                    <small>Flag for the applicable additional controls.</small>
                  </span>
                </label>

                <label class="pfm-setting">
                  <span class="pfm-checkbox">
                    <input
                      v-model="form.is_active"
                      type="checkbox"
                    />

                    <Check :size="13" />
                  </span>

                  <span class="pfm-setting-copy">
                    <strong>Active product</strong>
                    <small>Available for normal product workflows.</small>
                  </span>

                  <span
                    class="pfm-active-badge"
                    :class="{ inactive: !form.is_active }"
                  >
                    {{ form.is_active ? 'Active' : 'Inactive' }}
                  </span>
                </label>
              </div>
            </section>

            <div v-if="errorMessage" class="pfm-error">
              <span class="pfm-error-icon">!</span>
              <span>{{ errorMessage }}</span>
            </div>

            <footer class="pfm-footer">
              <span class="pfm-required-note">
                <b>*</b> Required fields
              </span>

              <div class="pfm-footer-actions">
                <button
                  type="button"
                  class="pfm-button pfm-secondary"
                  :disabled="saving"
                  @click="emit('close')"
                >
                  Cancel
                </button>

                <button
                  type="submit"
                  class="pfm-button pfm-primary"
                  :disabled="saving"
                >
                  <LoaderCircle
                    v-if="saving"
                    :size="16"
                    class="pfm-spin"
                  />

                  <Check v-else :size="16" />

                  {{
                    saving
                      ? 'Saving...'
                      : mode === 'create'
                        ? 'Create product'
                        : 'Save changes'
                  }}
                </button>
              </div>
            </footer>
          </form>

          <!-- Add Brand / Composition / HSN popup -->
          <Transition name="pfm-fade">
            <div
              v-if="activePopup"
              class="pfm-popup-backdrop"
              @click.self="closeAddPopup"
            >
              <section class="pfm-popup" role="dialog" aria-modal="true">
                <header class="pfm-popup-header">
                  <div class="pfm-popup-icon">
                    <Plus :size="19" />
                  </div>

                  <div>
                    <h3>
                      {{
                        activePopup === 'brand'
                          ? 'Add new brand'
                          : activePopup === 'composition'
                            ? 'Add composition'
                            : 'Add New HSN'
                      }}
                    </h3>

                    <p>
                      {{
                        activePopup === 'hsn'
                          ? 'Create a record in HSN Master.'
                          : 'Create a new master record.'
                      }}
                    </p>
                  </div>

                  <button
                    type="button"
                    class="pfm-close"
                    aria-label="Close popup"
                    @click="closeAddPopup"
                  >
                    <X :size="17" />
                  </button>
                </header>

                <!-- HSN creation form -->
                <form
                  v-if="activePopup === 'hsn'"
                  class="pfm-popup-form"
                  @submit.prevent="submitNewHsn"
                >
                  <label class="pfm-field">
                    <span>HSN code <b>*</b></span>

                    <input
                      v-model="newHsnCode"
                      autofocus
                      required
                      inputmode="numeric"
                      maxlength="8"
                      placeholder="e.g. 30049099"
                    />
                  </label>

                  <label class="pfm-field">
                    <span>Description</span>

                    <input
                      v-model="newHsnDescription"
                      maxlength="500"
                      placeholder="Enter HSN description"
                    />
                  </label>

                  <label class="pfm-field">
                    <span>GST rate (%) <b>*</b></span>

                    <div class="pfm-input-suffix">
                      <input
                        v-model.number="newHsnGstRate"
                        type="number"
                        min="0"
                        max="100"
                        step="0.01"
                        required
                      />

                      <span>%</span>
                    </div>
                  </label>

                  <p v-if="hsnError" class="pfm-popup-error">
                    {{ hsnError }}
                  </p>

                  <footer class="pfm-popup-footer">
                    <button
                      type="button"
                      class="pfm-button pfm-secondary"
                      :disabled="hsnSaving"
                      @click="closeAddPopup"
                    >
                      Cancel
                    </button>

                    <button
                      type="submit"
                      class="pfm-button pfm-primary"
                      :disabled="hsnSaving"
                    >
                      <LoaderCircle
                        v-if="hsnSaving"
                        :size="15"
                        class="pfm-spin"
                      />

                      <Plus v-else :size="15" />

                      {{ hsnSaving ? 'Saving...' : 'Save HSN' }}
                    </button>
                  </footer>
                </form>

                <!-- Brand / Composition form -->
                <form
                  v-else
                  class="pfm-popup-form"
                  @submit.prevent="submitNewMaster"
                >
                  <label class="pfm-field">
                    <span>
                      {{ activePopup === 'brand' ? 'Brand name' : 'Composition name' }}
                      <b>*</b>
                    </span>

                    <input
                      v-model="popupName"
                      autofocus
                      required
                      maxlength="500"
                      :placeholder="
                        activePopup === 'brand'
                          ? 'e.g. Cipla'
                          : 'e.g. Paracetamol'
                      "
                    />
                  </label>

                  <p v-if="popupError" class="pfm-popup-error">
                    {{ popupError }}
                  </p>

                  <footer class="pfm-popup-footer">
                    <button
                      type="button"
                      class="pfm-button pfm-secondary"
                      @click="closeAddPopup"
                    >
                      Cancel
                    </button>

                    <button
                      type="submit"
                      class="pfm-button pfm-primary"
                      :disabled="
                        !popupName.trim() ||
                        (activePopup === 'brand'
                          ? addingBrand
                          : addingComposition)
                      "
                    >
                      <LoaderCircle
                        v-if="
                          activePopup === 'brand'
                            ? addingBrand
                            : addingComposition
                        "
                        :size="15"
                        class="pfm-spin"
                      />

                      <Plus v-else :size="15" />

                      Add {{ activePopup === 'brand' ? 'brand' : 'composition' }}
                    </button>
                  </footer>
                </form>
              </section>
            </div>
          </Transition>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<style>
.pfm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgb(17 24 39 / 46%);
  backdrop-filter: blur(4px);
  font-family: inherit;
}

.pfm-modal,
.pfm-popup {
  --pfm-green: #16835f;
  --pfm-border: #e2e8e4;
  --pfm-text: #202a25;
  --pfm-muted: #78847d;

  color: var(--pfm-text);
}

.pfm-modal {
  position: relative;
  width: min(760px, 100%);
  max-height: min(92vh, 940px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #fff;
  border: 1px solid #e3e9e5;
  border-radius: 14px;
  box-shadow: 0 24px 80px rgb(0 0 0 / 22%);
}

.pfm-header {
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 21px 24px;
  border-bottom: 1px solid #edf0ee;
}

.pfm-heading-icon {
  width: 43px;
  height: 43px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: #16835f;
  background: #eaf7f0;
  border: 1px solid #d5efdf;
  border-radius: 8px;
}

.pfm-heading-copy {
  flex: 1;
  min-width: 0;
}

.pfm-eyebrow {
  color: #16835f;
  font-size: 10px;
  font-weight: 750;
  letter-spacing: .1em;
}

.pfm-header h2 {
  margin: 4px 0;
  color: #17231c;
  font-size: 23px;
  font-weight: 700;
  letter-spacing: -.035em;
}

.pfm-header p {
  margin: 0;
  color: #78847d;
  font-size: 12px;
}

.pfm-close {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: #69756d;
  background: white;
  border: 1px solid #e2e8e4;
  border-radius: 8px;
  cursor: pointer;
  transition: background .15s, color .15s;
}

.pfm-close:hover {
  color: #17231c;
  background: #f2f7f3;
}

.pfm-form {
  overflow-y: auto;
  padding: 23px 24px 24px;
  scrollbar-width: thin;
  scrollbar-color: #cbd5ce transparent;
}

.pfm-section + .pfm-section {
  margin-top: 28px;
  padding-top: 24px;
  border-top: 1px solid #edf0ee;
}

.pfm-section-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 19px;
}

.pfm-section-icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  border-radius: 8px;
}

.pfm-section-icon.green {
  color: #16835f;
  background: #eaf7f0;
}

.pfm-section-icon.purple {
  color: #7854b7;
  background: #f2edfc;
}

.pfm-section-icon.amber {
  color: #a46b16;
  background: #fff5df;
}

.pfm-section-heading h3 {
  margin: 0;
  color: #26342b;
  font-size: 13px;
  font-weight: 750;
}

.pfm-section-heading p {
  margin: 3px 0 0;
  color: #879189;
  font-size: 11px;
}

.pfm-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 17px 15px;
}

.pfm-field {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 7px;
  min-width: 0;
  color: #37443b;
  font-size: 12px;
  font-weight: 650;
}

.pfm-field > span:first-child b {
  color: #dc5555;
}

.pfm-full {
  grid-column: 1 / -1;
}

.pfm-field input,
.pfm-field select,
.pfm-input-suffix input,
.pfm-input-prefix input {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  height: 40px;
  padding: 0 12px;
  color: #253229;
  background: white;
  border: 1px solid #dce5de;
  border-radius: 8px;
  outline: none;
  font-family: inherit;
  font-size: 12px;
  font-weight: 400;
  transition: border-color .15s, box-shadow .15s, background .15s;
}

.pfm-field input::placeholder,
.pfm-menu-search input::placeholder {
  color: #9ba59e;
}

.pfm-field input:hover,
.pfm-field select:hover {
  border-color: #bdcec1;
}

.pfm-field input:focus,
.pfm-field select:focus {
  border-color: #249b70;
  box-shadow: 0 0 0 3px rgb(22 131 95 / 11%);
}

.pfm-field select {
  cursor: pointer;
}

.pfm-master-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.pfm-dropdown {
  position: relative;
  flex: 1;
  min-width: 0;
}

.pfm-select-trigger {
  box-sizing: border-box;
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 12px;
  color: #253229;
  background: white;
  border: 1px solid #dce5de;
  border-radius: 8px;
  text-align: left;
  font-family: inherit;
  font-size: 12px;
  font-weight: 450;
  cursor: pointer;
  transition: border-color .15s, box-shadow .15s, background .15s;
}

.pfm-select-trigger:hover,
.pfm-select-trigger.is-open {
  border-color: #249b70;
}

.pfm-select-trigger.is-open {
  box-shadow: 0 0 0 3px rgb(22 131 95 / 9%);
}

.pfm-trigger-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pfm-trigger-value.muted {
  color: #8d9990;
}

.pfm-chevron {
  flex-shrink: 0;
  color: #748278;
  transition: transform .18s;
}

.pfm-chevron.rotated {
  transform: rotate(180deg);
}

.pfm-add-button {
  width: 40px;
  height: 40px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: #16835f;
  background: #f0faf4;
  border: 1px solid #cce8d7;
  border-radius: 8px;
  cursor: pointer;
  transition: background .15s, border-color .15s, transform .15s;
}

.pfm-add-button:hover {
  background: #dff5e8;
  border-color: #9dd7b5;
}

.pfm-add-button:active {
  transform: scale(.96);
}

.pfm-dropdown-menu {
  position: absolute;
  z-index: 40;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  overflow: hidden;
  background: white;
  border: 1px solid #dce7df;
  border-radius: 8px;
  box-shadow:
    0 12px 30px rgb(23 55 36 / 13%),
    0 2px 5px rgb(23 55 36 / 4%);
}

.pfm-menu-search {
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 8px;
  padding: 0 10px;
  color: #8a978e;
  background: #f7faf8;
  border: 1px solid #e5ece7;
  border-radius: 8px;
}

.pfm-menu-search:focus-within {
  color: #16835f;
  background: white;
  border-color: #75bd97;
  box-shadow: 0 0 0 2px rgb(22 131 95 / 8%);
}

.pfm-menu-search input {
  flex: 1;
  min-width: 0;
  height: 35px;
  padding: 0;
  color: #26342b;
  background: transparent;
  border: 0;
  outline: 0;
  font-family: inherit;
  font-size: 12px;
}

.pfm-menu-search input:focus {
  border: 0;
  box-shadow: none;
}

.pfm-menu-list {
  max-height: 230px;
  overflow-y: auto;
  padding: 2px 6px 6px;
  scrollbar-width: thin;
  scrollbar-color: #cbd5ce transparent;
}

.pfm-option {
  box-sizing: border-box;
  width: 100%;
  min-height: 43px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 9px;
  color: #36443a;
  background: white;
  border: 0;
  border-radius: 8px;
  text-align: left;
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: background .12s;
}

.pfm-option:hover {
  background: #f2f8f4;
}

.pfm-option.is-selected {
  color: #146e50;
  background: #eaf7f0;
}

.pfm-option-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.pfm-option-name {
  overflow: hidden;
  color: inherit;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 550;
}

.pfm-option-sub {
  overflow: hidden;
  color: #8b968e;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pfm-option-check {
  flex-shrink: 0;
  color: #16835f;
}

.pfm-hsn-rate {
  flex-shrink: 0;
  padding: 4px 6px;
  color: #42634f;
  background: #edf7ef;
  border-radius: 5px;
  font-size: 10px;
  font-weight: 700;
}

.pfm-empty {
  min-height: 75px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 12px;
  color: #8a978e;
  font-size: 12px;
  text-align: center;
}

.pfm-inline-error {
  padding: 10px;
  color: #a72e38;
  font-size: 11px;
  line-height: 1.5;
}

.pfm-menu-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 9px 12px;
  background: #fafcfb;
  border-top: 1px solid #eaf0eb;
}

.pfm-menu-footer > span {
  color: #87938a;
  font-size: 10px;
}

.pfm-menu-footer button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 7px;
  color: #16835f;
  background: transparent;
  border: 0;
  border-radius: 8px;
  font-family: inherit;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.pfm-menu-footer button:hover {
  background: #eaf7f0;
}

.pfm-help {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: #89958c;
  font-size: 10px;
  font-weight: 450;
}

.pfm-input-suffix,
.pfm-input-prefix {
  position: relative;
  display: flex;
  align-items: center;
}

.pfm-input-suffix input {
  padding-right: 35px;
}

.pfm-input-suffix > span {
  position: absolute;
  right: 12px;
  color: #7f8c82;
  font-size: 12px;
}

.pfm-input-prefix {
  gap: 10px;
  padding-left: 11px;
  color: #7d8b80;
  border: 1px solid #dce5de;
  border-radius: 8px;
}

.pfm-input-prefix:focus-within {
  border-color: #249b70;
  box-shadow: 0 0 0 3px rgb(22 131 95 / 11%);
}

.pfm-input-prefix input {
  border: 0;
  box-shadow: none;
}

.pfm-input-prefix input:focus {
  border: 0;
  box-shadow: none;
}

.pfm-schedules {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}

.pfm-schedule {
  position: relative;
  min-width: 0;
  min-height: 70px;
  display: flex;
  align-items: flex-start;
  gap: 9px;
  padding: 12px 10px;
  background: #fff;
  border: 1px solid #e1e8e2;
  border-radius: 8px;
  cursor: pointer;
  transition: background .15s, border-color .15s, box-shadow .15s;
}

.pfm-schedule input {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.pfm-schedule-check {
  width: 17px;
  height: 17px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: transparent;
  background: white;
  border: 1px solid #cbd6ce;
  border-radius: 5px;
}

.pfm-schedule-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.pfm-schedule-copy strong {
  color: #37443b;
  font-size: 11px;
  font-weight: 750;
}

.pfm-schedule-copy small {
  color: #87938a;
  font-size: 9px;
  line-height: 1.45;
}

.pfm-schedule:hover {
  background: #f9fcfa;
  border-color: #a9cdb6;
}

.pfm-schedule.is-selected {
  background: #eff9f2;
  border-color: #55ae7c;
  box-shadow: 0 0 0 1px rgb(22 131 95 / 8%);
}

.pfm-schedule.is-selected .pfm-schedule-check {
  color: white;
  background: #16835f;
  border-color: #16835f;
}

.schedule-amber.is-selected {
  background: #fff8e9;
  border-color: #e4bd65;
}

.schedule-amber.is-selected .pfm-schedule-check {
  background: #b78320;
  border-color: #b78320;
}

.schedule-orange.is-selected {
  background: #fff4eb;
  border-color: #e7a16c;
}

.schedule-orange.is-selected .pfm-schedule-check {
  background: #c66c29;
  border-color: #c66c29;
}

.schedule-rose.is-selected {
  background: #fff0f1;
  border-color: #e7a0a6;
}

.schedule-rose.is-selected .pfm-schedule-check {
  background: #c54857;
  border-color: #c54857;
}

.pfm-settings {
  display: grid;
  gap: 9px;
  margin-top: 18px;
}

.pfm-setting {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 12px;
  background: #f8faf8;
  border: 1px solid #e9eeea;
  border-radius: 8px;
  cursor: pointer;
  transition: background .15s, border-color .15s;
}

.pfm-setting:hover {
  background: #f1f8f3;
  border-color: #d1e7d7;
}

.pfm-checkbox {
  position: relative;
  width: 18px;
  height: 18px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: white;
  background: white;
  border: 1px solid #c8d4cb;
  border-radius: 5px;
}

.pfm-checkbox input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  margin: 0;
  opacity: 0;
  cursor: pointer;
}

.pfm-checkbox svg {
  opacity: 0;
}

.pfm-checkbox:has(input:checked) {
  background: #16835f;
  border-color: #16835f;
}

.pfm-checkbox:has(input:checked) svg {
  opacity: 1;
}

.pfm-setting-copy {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pfm-setting-copy strong {
  color: #354239;
  font-size: 11px;
  font-weight: 700;
}

.pfm-setting-copy small {
  color: #87938a;
  font-size: 10px;
  line-height: 1.4;
}

.pfm-active-badge {
  padding: 4px 8px;
  color: #16754e;
  background: #e2f6e9;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 750;
}

.pfm-active-badge.inactive {
  color: #6f7771;
  background: #e9ecea;
}

.pfm-error {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  margin-top: 18px;
  padding: 12px;
  color: #a72e38;
  background: #fff1f1;
  border: 1px solid #f3d3d5;
  border-radius: 8px;
  font-size: 12px;
}

.pfm-error-icon {
  width: 18px;
  height: 18px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: white;
  background: #c54857;
  border-radius: 50%;
  font-size: 12px;
  font-weight: 800;
}

.pfm-footer {
  position: sticky;
  bottom: -24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin: 26px -24px -24px;
  padding: 16px 24px;
  background: rgb(255 255 255 / 96%);
  border-top: 1px solid #e9eeea;
  backdrop-filter: blur(10px);
}

.pfm-required-note {
  color: #87938a;
  font-size: 10px;
}

.pfm-required-note b {
  color: #dc5555;
}

.pfm-footer-actions,
.pfm-popup-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 9px;
}

.pfm-button {
  min-height: 39px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 14px;
  border: 1px solid transparent;
  border-radius: 8px;
  font-family: inherit;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: background .15s, border-color .15s, transform .15s;
}

.pfm-button:active:not(:disabled) {
  transform: translateY(1px);
}

.pfm-button:disabled {
  opacity: .55;
  cursor: not-allowed;
}

.pfm-secondary {
  color: #455249;
  background: white;
  border-color: #dce5de;
}

.pfm-secondary:hover:not(:disabled) {
  background: #f5f8f5;
  border-color: #c3d1c6;
}

.pfm-primary {
  color: white;
  background: #16835f;
  border-color: #16835f;
  box-shadow: 0 2px 5px rgb(22 131 95 / 13%);
}

.pfm-primary:hover:not(:disabled) {
  background: #116849;
  border-color: #116849;
}

.pfm-popup-backdrop {
  position: absolute;
  z-index: 100;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgb(18 35 25 / 28%);
  backdrop-filter: blur(2px);
}

.pfm-popup {
  width: min(400px, 100%);
  max-height: 90%;
  overflow-y: auto;
  padding: 21px;
  background: white;
  border: 1px solid #e2e9e3;
  border-radius: 12px;
  box-shadow: 0 22px 65px rgb(0 0 0 / 18%);
}

.pfm-popup-header {
  display: flex;
  align-items: center;
  gap: 11px;
  margin-bottom: 22px;
}

.pfm-popup-icon {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  color: #16835f;
  background: #eaf7f0;
  border-radius: 8px;
}

.pfm-popup-header > div:nth-child(2) {
  flex: 1;
  min-width: 0;
}

.pfm-popup-header h3 {
  margin: 0;
  color: #233027;
  font-size: 16px;
  font-weight: 750;
}

.pfm-popup-header p {
  margin: 4px 0 0;
  color: #87938a;
  font-size: 11px;
  line-height: 1.5;
}

.pfm-popup-form {
  display: grid;
  gap: 18px;
}

.pfm-popup-error {
  margin: -10px 0 0;
  color: #b63742;
  font-size: 11px;
  line-height: 1.5;
}

.pfm-popup-footer {
  padding-top: 15px;
  border-top: 1px solid #edf0ee;
}

.pfm-spin {
  animation: pfm-spin 1s linear infinite;
}

.pfm-fade-enter-active,
.pfm-fade-leave-active {
  transition: opacity .16s ease;
}

.pfm-fade-enter-from,
.pfm-fade-leave-to {
  opacity: 0;
}

.pfm-menu-enter-active,
.pfm-menu-leave-active {
  transform-origin: top;
  transition: opacity .12s ease, transform .12s ease;
}

.pfm-menu-enter-from,
.pfm-menu-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(.99);
}

@keyframes pfm-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 620px) {
  .pfm-backdrop {
    padding: 7px;
  }

  .pfm-modal {
    max-height: 96vh;
    border-radius: 12px;
  }

  .pfm-header {
    gap: 10px;
    padding: 16px;
  }

  .pfm-heading-icon {
    width: 37px;
    height: 37px;
  }

  .pfm-header h2 {
    font-size: 19px;
  }

  .pfm-header p {
    font-size: 11px;
  }

  .pfm-form {
    padding: 18px 16px 20px;
  }

  .pfm-grid {
    grid-template-columns: 1fr;
    gap: 15px;
  }

  .pfm-full {
    grid-column: auto;
  }

  .pfm-schedules {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .pfm-footer {
    bottom: -20px;
    align-items: flex-end;
    margin: 23px -16px -20px;
    padding: 13px 16px;
  }

  .pfm-required-note {
    max-width: 75px;
    line-height: 1.5;
  }

  .pfm-button {
    padding: 0 10px;
    font-size: 11px;
  }

  .pfm-popup {
    padding: 17px;
  }
}
</style>