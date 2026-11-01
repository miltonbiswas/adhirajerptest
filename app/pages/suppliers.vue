<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Party } from '~/composables/useApi'
import PartyFormModal from '~/components/parties/PartyFormModal.vue'

useSeoMeta({ title: 'Suppliers | AdhirajPOS' })

const { getParties, deactivateParty } = useApi()

const suppliers = ref<Party[]>([])
const loading = ref(false)
const actionLoading = ref<string | null>(null)
const pageError = ref('')
const successMessage = ref('')
const search = ref('')
const activeFilter = ref<'all' | 'active' | 'inactive'>('active')
const page = ref(1)
const pageSize = 25
const total = ref(0)

const modalOpen = ref(false)
const selectedParty = ref<Party | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | undefined
let messageTimer: ReturnType<typeof setTimeout> | undefined
let requestSequence = 0

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / pageSize)),
)

const startRecord = computed(() =>
  total.value > 0 ? (page.value - 1) * pageSize + 1 : 0,
)

const endRecord = computed(() =>
  Math.min(page.value * pageSize, total.value),
)

const formatCurrency = (value: number | string | null | undefined) => {
  const amount = Number(value ?? 0)

  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2,
  }).format(Number.isFinite(amount) ? amount : 0)
}

const formatLocation = (party: Party) =>
  [party.city, party.state].filter(Boolean).join(', ') || '—'

const formatBalanceType = (party: Party) => {
  switch (party.opening_balance_type) {
    case 'PAYABLE':
      return 'Payable'
    case 'RECEIVABLE':
      return 'Receivable'
    case 'ADVANCE':
      return 'Advance'
    default:
      return 'Opening balance'
  }
}

function showMessage(message: string, type: 'success' | 'error') {
  if (messageTimer) clearTimeout(messageTimer)

  if (type === 'success') {
    successMessage.value = message
    pageError.value = ''
  } else {
    pageError.value = message
    successMessage.value = ''
  }

  messageTimer = setTimeout(() => {
    successMessage.value = ''
    pageError.value = ''
  }, 5000)
}

async function loadSuppliers() {
  const sequence = ++requestSequence

  loading.value = true
  pageError.value = ''

  try {
    const response = await getParties({
      search: search.value.trim() || undefined,
      role: 'SUPPLIER',
      page: page.value,
      limit: pageSize,
      active:
        activeFilter.value === 'all'
          ? undefined
          : activeFilter.value === 'active',
    })

    if (sequence !== requestSequence) return

    suppliers.value = response.data ?? []
    total.value = Number(response.total ?? 0)

    const lastPage = Math.max(1, Math.ceil(total.value / pageSize))

    if (page.value > lastPage) {
      page.value = lastPage
    }
  } catch (error: unknown) {
    if (sequence !== requestSequence) return

    suppliers.value = []
    total.value = 0

    pageError.value =
      error instanceof Error
        ? error.message
        : 'Unable to load suppliers. Please try again.'
  } finally {
    if (sequence === requestSequence) {
      loading.value = false
    }
  }
}

function handleSearch() {
  if (searchTimer) clearTimeout(searchTimer)

  searchTimer = setTimeout(() => {
    if (page.value !== 1) {
      page.value = 1
    } else {
      void loadSuppliers()
    }
  }, 300)
}

function resetFilters() {
  if (searchTimer) clearTimeout(searchTimer)

  search.value = ''
  activeFilter.value = 'active'
  page.value = 1

  void loadSuppliers()
}

function openCreateModal() {
  selectedParty.value = null
  modalOpen.value = true
}

function openEditModal(party: Party) {
  selectedParty.value = party
  modalOpen.value = true
}

async function handleSaved() {
  modalOpen.value = false
  selectedParty.value = null

  showMessage('Supplier saved successfully.', 'success')
  await loadSuppliers()
}

async function handleDeactivate(party: Party) {
  if (!party.is_active || actionLoading.value) return

  const confirmed = window.confirm(
    `Deactivate "${party.name}"?\n\nThe supplier will remain in your records but will no longer appear in the active directory.`,
  )

  if (!confirmed) return

  actionLoading.value = party.id
  pageError.value = ''
  successMessage.value = ''

  try {
    await deactivateParty(party.id)

    showMessage('Supplier deactivated successfully.', 'success')

    if (suppliers.value.length === 1 && page.value > 1) {
      page.value -= 1
    } else {
      await loadSuppliers()
    }
  } catch (error: unknown) {
    showMessage(
      error instanceof Error
        ? error.message
        : 'Unable to deactivate this supplier.',
      'error',
    )
  } finally {
    actionLoading.value = null
  }
}

function changePage(nextPage: number) {
  if (nextPage < 1 || nextPage > totalPages.value || loading.value) return
  page.value = nextPage
}

watch([activeFilter, page], () => {
  void loadSuppliers()
})

onMounted(() => {
  void loadSuppliers()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  if (messageTimer) clearTimeout(messageTimer)
  requestSequence++
})
</script>

<template>
  <main class="suppliers-page">
    <header class="page-heading">
      <div class="heading-copy">
        <div class="section-eyebrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <rect x="3" y="7" width="18" height="13" rx="2" />
            <path d="M8 7V4h8v3M3 12h18M9 12v2h6v-2" />
          </svg>
          <span>PARTY MANAGEMENT</span>
        </div>

        <h1>Suppliers</h1>
        <p>Manage supplier records, contact details and business information.</p>
      </div>

      <div class="heading-actions">
        <button
          class="secondary-button"
          type="button"
          :disabled="loading"
          @click="loadSuppliers"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <path d="M20 7v5h-5M4 17v-5h5" />
            <path d="M5.6 9A7 7 0 0 1 18 6l2 6M18.4 15A7 7 0 0 1 6 18l-2-6" />
          </svg>
          Refresh
        </button>

        <button class="primary-button" type="button" @click="openCreateModal">
          <span class="button-icon">+</span>
          Add supplier
        </button>
      </div>
    </header>

    <div v-if="successMessage" class="notice notice-success" role="status">
      <span>{{ successMessage }}</span>
      <button type="button" aria-label="Dismiss notification" @click="successMessage = ''">×</button>
    </div>

    <div v-if="pageError" class="notice notice-error" role="alert">
      <span>{{ pageError }}</span>
      <button type="button" aria-label="Dismiss error" @click="pageError = ''">×</button>
    </div>

    <section class="summary-grid" aria-label="Supplier summary">
      <article class="summary-card">
        <span class="summary-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <rect x="3" y="7" width="18" height="13" rx="2" />
            <path d="M8 7V4h8v3M3 12h18" />
          </svg>
        </span>
        <div class="summary-content">
          <span class="summary-label">Matching suppliers</span>
          <strong>{{ loading && total === 0 ? '—' : total.toLocaleString('en-IN') }}</strong>
        </div>
      </article>

      <article class="summary-card">
        <span class="summary-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <path d="m5 12 4 4L19 6" />
          </svg>
        </span>
        <div class="summary-content">
          <span class="summary-label">Active on page</span>
          <strong>
            {{ suppliers.filter(supplier => supplier.is_active).length.toLocaleString('en-IN') }}
          </strong>
        </div>
      </article>

      <article class="summary-card">
        <span class="summary-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="2" />
            <path d="M8 9h8M8 13h8M8 17h4" />
          </svg>
        </span>
        <div class="summary-content">
          <span class="summary-label">Records on page</span>
          <strong>{{ suppliers.length.toLocaleString('en-IN') }}</strong>
        </div>
      </article>
    </section>

    <section class="directory-card">
      <div class="directory-titlebar">
        <div>
          <h2>Supplier register</h2>
          <p>Search, review and manage supplier accounts.</p>
        </div>

        <span class="record-count">
          {{ total.toLocaleString('en-IN') }} records
        </span>
      </div>

      <div class="directory-toolbar">
        <label class="search-field">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <circle cx="11" cy="11" r="7" />
            <path d="m16 16 4 4" />
          </svg>
          <span class="sr-only">Search suppliers</span>
          <input
            v-model="search"
            type="search"
            placeholder="Search supplier, code, phone or GSTIN..."
            @input="handleSearch"
          />
        </label>

        <label class="select-field">
          <span class="sr-only">Filter supplier status</span>
          <select v-model="activeFilter">
            <option value="active">Active suppliers</option>
            <option value="inactive">Inactive suppliers</option>
            <option value="all">All suppliers</option>
          </select>
        </label>

        <button
          class="reset-button"
          type="button"
          :disabled="loading && !search && activeFilter === 'active'"
          @click="resetFilters"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <path d="M3 12a9 9 0 1 0 2.6-6.4L3 8" />
            <path d="M3 3v5h5" />
          </svg>
          Reset
        </button>
      </div>

      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th>Supplier</th>
              <th>Contact</th>
              <th>GSTIN</th>
              <th>Location</th>
              <th class="numeric-cell">Opening balance</th>
              <th>Status</th>
              <th class="actions-heading">Actions</th>
            </tr>
          </thead>

          <tbody>
            <tr v-if="loading && suppliers.length === 0">
              <td colspan="7" class="state-cell">
                <span class="loading-indicator" />
                Loading suppliers...
              </td>
            </tr>

            <tr v-else-if="suppliers.length === 0">
              <td colspan="7" class="state-cell">
                <div class="empty-state">
                  <span class="empty-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                      <rect x="3" y="7" width="18" height="13" rx="2" />
                      <path d="M8 7V4h8v3M3 12h18" />
                    </svg>
                  </span>

                  <strong>
                    {{ search.trim() ? 'No matching suppliers' : 'No suppliers found' }}
                  </strong>

                  <p>
                    {{
                      search.trim()
                        ? 'Try another search term or change the status filter.'
                        : activeFilter === 'inactive'
                          ? 'There are no inactive suppliers matching these filters.'
                          : 'Add a supplier to start building your directory.'
                    }}
                  </p>

                  <button
                    v-if="!search.trim() && activeFilter !== 'inactive'"
                    type="button"
                    class="secondary-button"
                    @click="openCreateModal"
                  >
                    Add your first supplier
                  </button>
                </div>
              </td>
            </tr>

            <tr v-for="supplier in suppliers" v-else :key="supplier.id">
              <td>
                <div class="supplier-cell">
                  <span class="supplier-avatar">
                    {{ supplier.name.trim().charAt(0).toUpperCase() || 'S' }}
                  </span>
                  <div class="supplier-identity">
                    <button
                      type="button"
                      class="supplier-name"
                      @click="openEditModal(supplier)"
                    >
                      {{ supplier.name }}
                    </button>
                    <span>{{ supplier.party_code || 'No code assigned' }}</span>
                  </div>
                </div>
              </td>

              <td>
                <div class="contact-cell">
                  <span>{{ supplier.phone || '—' }}</span>
                  <small>{{ supplier.email || supplier.contact_person || '—' }}</small>
                </div>
              </td>

              <td>
                <span class="muted-text">{{ supplier.gstin || '—' }}</span>
              </td>

              <td>
                <div class="location-cell">
                  <span>{{ formatLocation(supplier) }}</span>
                  <small v-if="supplier.pincode">{{ supplier.pincode }}</small>
                </div>
              </td>

              <td class="numeric-cell">
                <div class="balance-cell">
                  <strong>{{ formatCurrency(supplier.opening_balance) }}</strong>
                  <small>{{ formatBalanceType(supplier) }}</small>
                </div>
              </td>

              <td>
                <span
                  class="status-badge"
                  :class="supplier.is_active ? 'status-active' : 'status-inactive'"
                >
                  <span class="status-dot" />
                  {{ supplier.is_active ? 'Active' : 'Inactive' }}
                </span>
              </td>

              <td>
                <div class="row-actions">
                  <button
                    type="button"
                    class="row-action"
                    :aria-label="`Edit ${supplier.name}`"
                    @click="openEditModal(supplier)"
                  >
                    Edit
                  </button>

                  <button
                    v-if="supplier.is_active"
                    type="button"
                    class="row-action row-action-danger"
                    :disabled="actionLoading === supplier.id"
                    :aria-label="`Deactivate ${supplier.name}`"
                    @click="handleDeactivate(supplier)"
                  >
                    {{ actionLoading === supplier.id ? 'Saving...' : 'Deactivate' }}
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <footer class="directory-footer">
        <span>
          Showing <strong>{{ startRecord }}–{{ endRecord }}</strong>
          of <strong>{{ total.toLocaleString('en-IN') }}</strong> suppliers
        </span>

        <div class="pagination">
          <button
            type="button"
            class="pagination-button"
            :disabled="page <= 1 || loading"
            @click="changePage(page - 1)"
          >
            Previous
          </button>

          <span class="page-indicator">{{ page }} / {{ totalPages }}</span>

          <button
            type="button"
            class="pagination-button"
            :disabled="page >= totalPages || loading"
            @click="changePage(page + 1)"
          >
            Next
          </button>
        </div>
      </footer>
    </section>

    <PartyFormModal
      :open="modalOpen"
      :party="selectedParty"
      default-role="SUPPLIER"
      @close="modalOpen = false"
      @saved="handleSaved"
    />
  </main>
</template>

<style scoped>
.suppliers-page {
  min-height: 100%;
  padding: 22px 24px 30px;
  color: #171717;
  font-size: 13px;
}

.page-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 18px;
}

.heading-copy {
  min-width: 0;
}

.section-eyebrow {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 7px;
  color: #737373;
  font-size: 11px;
  font-weight: 650;
  letter-spacing: 0.12em;
}

.section-eyebrow svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.page-heading h1 {
  margin: 0;
  font-size: 27px;
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: -0.045em;
}

.page-heading p,
.directory-titlebar p {
  margin: 6px 0 0;
  color: #737373;
  font-size: 12px;
  line-height: 1.5;
}

.heading-actions {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 8px;
}

.primary-button,
.secondary-button,
.reset-button,
.pagination-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 36px;
  padding: 0 13px;
  border: 1px solid #d4d4d4;
  border-radius: 6px;
  background: #fff;
  color: #262626;
  font-size: 12px;
  font-weight: 550;
  white-space: nowrap;
  transition: background 120ms ease, border-color 120ms ease;
}

.primary-button {
  border-color: #171717;
  background: #171717;
  color: #fff;
}

.primary-button:hover {
  border-color: #404040;
  background: #404040;
}

.secondary-button:hover,
.reset-button:hover:not(:disabled),
.pagination-button:hover:not(:disabled) {
  border-color: #a3a3a3;
  background: #f5f5f5;
}

.primary-button:disabled,
.secondary-button:disabled,
.reset-button:disabled,
.pagination-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.primary-button svg,
.secondary-button svg,
.reset-button svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.button-icon {
  font-size: 18px;
  font-weight: 400;
  line-height: 1;
}

.notice {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
  padding: 11px 14px;
  border: 1px solid;
  border-radius: 6px;
  font-size: 12px;
}

.notice > button {
  padding: 0 3px;
  font-size: 20px;
  line-height: 1;
}

.notice-success {
  border-color: #a7f3d0;
  background: #ecfdf5;
  color: #065f46;
}

.notice-error {
  border-color: #fecaca;
  background: #fef2f2;
  color: #991b1b;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 14px;
}

.summary-card {
  display: flex;
  min-width: 0;
  min-height: 66px;
  align-items: center;
  gap: 11px;
  padding: 11px 13px;
  border: 1px solid #e5e5e5;
  border-radius: 7px;
  background: #fff;
}

.summary-icon {
  display: flex;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  background: #f5f5f5;
  color: #525252;
}

.summary-icon svg {
  width: 17px;
  height: 17px;
}

.summary-content {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
}

.summary-label {
  color: #737373;
  font-size: 11px;
  line-height: 1.4;
}

.summary-content strong {
  font-size: 19px;
  font-weight: 700;
  line-height: 1.2;
  font-variant-numeric: tabular-nums;
}

.directory-card {
  overflow: hidden;
  border: 1px solid #dedede;
  border-radius: 8px;
  background: #fff;
}

.directory-titlebar {
  display: flex;
  min-height: 57px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 15px;
  border-bottom: 1px solid #e5e5e5;
}

.directory-titlebar h2 {
  margin: 0;
  font-size: 13px;
  font-weight: 700;
  line-height: 1.4;
}

.directory-titlebar p {
  margin-top: 3px;
  font-size: 11px;
}

.record-count {
  flex: 0 0 auto;
  padding: 5px 8px;
  border: 1px solid #e5e5e5;
  border-radius: 5px;
  color: #525252;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.directory-toolbar {
  display: grid;
  grid-template-columns: minmax(200px, 1fr) minmax(155px, 185px) auto;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid #e5e5e5;
}

.search-field {
  display: flex;
  min-width: 0;
  height: 36px;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  border: 1px solid #d4d4d4;
  border-radius: 6px;
  background: #fff;
  color: #737373;
}

.search-field:focus-within {
  border-color: #737373;
  box-shadow: 0 0 0 2px rgb(23 23 23 / 6%);
}

.search-field svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.search-field input {
  width: 100%;
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: #171717;
  font: inherit;
  font-size: 12px;
}

.search-field input::placeholder {
  color: #a3a3a3;
}

.search-field input::-webkit-search-cancel-button {
  cursor: pointer;
}

.select-field {
  display: block;
  min-width: 0;
}

.select-field select {
  width: 100%;
  height: 36px;
  padding: 0 28px 0 10px;
  border: 1px solid #d4d4d4;
  border-radius: 6px;
  outline: none;
  background-color: #fff;
  color: #404040;
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}

.select-field select:focus {
  border-color: #737373;
}

.reset-button {
  min-height: 36px;
  padding: 0 11px;
}

.table-scroll {
  width: 100%;
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  white-space: nowrap;
}

thead {
  background: #fafafa;
}

thead tr {
  border-bottom: 1px solid #e5e5e5;
}

th {
  padding: 11px 12px;
  color: #737373;
  font-size: 10px;
  font-weight: 650;
  letter-spacing: 0.065em;
  text-transform: uppercase;
}

td {
  padding: 11px 12px;
  border-bottom: 1px solid #ededed;
  color: #404040;
  font-size: 12px;
  vertical-align: middle;
}

tbody tr:last-child td {
  border-bottom: 0;
}

tbody tr:has(.supplier-cell):hover {
  background: #fafafa;
}

th:first-child,
td:first-child {
  padding-left: 15px;
}

.numeric-cell {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.actions-heading {
  text-align: right;
  padding-right: 15px;
}

.supplier-cell {
  display: flex;
  min-width: 155px;
  align-items: center;
  gap: 10px;
}

.supplier-avatar {
  display: flex;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  background: #f5f5f5;
  color: #525252;
  font-size: 13px;
  font-weight: 650;
}

.supplier-identity,
.contact-cell,
.location-cell,
.balance-cell {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.supplier-name {
  overflow: hidden;
  max-width: 250px;
  padding: 0;
  border: 0;
  background: transparent;
  color: #171717;
  font-size: 12px;
  font-weight: 650;
  text-align: left;
  text-overflow: ellipsis;
  cursor: pointer;
}

.supplier-name:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.supplier-identity > span,
.contact-cell small,
.location-cell small,
.balance-cell small {
  color: #737373;
  font-size: 10px;
  line-height: 1.4;
}

.contact-cell > span,
.location-cell > span {
  color: #262626;
  font-size: 12px;
}

.contact-cell small {
  overflow: hidden;
  max-width: 190px;
  text-overflow: ellipsis;
}

.muted-text {
  color: #525252;
  font-size: 11px;
}

.balance-cell strong {
  color: #171717;
  font-size: 12px;
  font-weight: 650;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border: 1px solid #d4d4d4;
  border-radius: 5px;
  font-size: 11px;
  font-weight: 550;
}

.status-active {
  border-color: #a7f3d0;
  background: #ecfdf5;
  color: #047857;
}

.status-inactive {
  background: #f5f5f5;
  color: #737373;
}

.status-dot {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: currentColor;
}

.row-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.row-action {
  padding: 4px 0;
  border: 0;
  background: transparent;
  color: #404040;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.row-action:hover {
  color: #000;
  text-decoration: underline;
  text-underline-offset: 3px;
}

.row-action-danger {
  color: #737373;
}

.row-action-danger:hover {
  color: #b91c1c;
}

.row-action:disabled {
  cursor: wait;
  opacity: 0.5;
}

.state-cell {
  height: 190px;
  padding: 28px 15px;
  color: #737373;
  text-align: center;
  font-size: 12px;
}

.loading-indicator {
  display: inline-block;
  width: 16px;
  height: 16px;
  margin-right: 9px;
  border: 2px solid #d4d4d4;
  border-top-color: #262626;
  border-radius: 50%;
  vertical-align: -4px;
  animation: spin 700ms linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-state {
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 8px;
}

.empty-icon {
  display: flex;
  width: 40px;
  height: 40px;
  align-items: center;
  justify-content: center;
  margin-bottom: 2px;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  background: #fafafa;
  color: #737373;
}

.empty-icon svg {
  width: 20px;
  height: 20px;
}

.empty-state strong {
  color: #262626;
  font-size: 13px;
  font-weight: 650;
}

.empty-state p {
  margin: 0 0 5px;
  color: #737373;
  font-size: 12px;
}

.directory-footer {
  display: flex;
  min-height: 53px;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 10px 15px;
  border-top: 1px solid #e5e5e5;
  color: #737373;
  font-size: 11px;
}

.directory-footer strong {
  color: #404040;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.pagination {
  display: flex;
  align-items: center;
  gap: 10px;
}

.pagination-button {
  min-height: 31px;
  padding: 0 11px;
  font-size: 11px;
}

.page-indicator {
  min-width: 44px;
  color: #525252;
  text-align: center;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  clip-path: inset(50%);
}

@media (min-width: 1600px) {
  .suppliers-page {
    padding-right: 28px;
    padding-left: 28px;
  }

  td {
    padding-top: 12px;
    padding-bottom: 12px;
  }
}

@media (max-width: 900px) {
  .suppliers-page {
    padding: 18px 16px 24px;
  }

  .directory-toolbar {
    grid-template-columns: minmax(180px, 1fr) minmax(145px, 180px) auto;
  }

  .summary-grid {
    gap: 8px;
  }
}

@media (max-width: 640px) {
  .suppliers-page {
    padding: 16px 12px 20px;
  }

  .page-heading {
    align-items: flex-start;
    flex-direction: column;
    gap: 13px;
  }

  .heading-actions {
    width: 100%;
  }

  .heading-actions button {
    flex: 1;
  }

  .page-heading h1 {
    font-size: 24px;
  }

  .summary-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .summary-card {
    min-height: 58px;
  }

  .directory-titlebar {
    align-items: flex-start;
  }

  .directory-toolbar {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .search-field {
    grid-column: 1 / -1;
  }

  .select-field {
    grid-column: 1;
  }

  .reset-button {
    grid-column: 2;
    grid-row: 2;
  }

  .directory-footer {
    align-items: flex-start;
    flex-direction: column;
  }

  .pagination {
    width: 100%;
    justify-content: space-between;
  }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
