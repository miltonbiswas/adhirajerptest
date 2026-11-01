<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Party } from '~/composables/useApi'
import PartyFormModal from '~/components/parties/PartyFormModal.vue'
useSeoMeta({ title: 'Customers | AdhirajPOS' })

const { getParties, deactivateParty } = useApi()

const customers = ref<Party[]>([])
const loading = ref(true)
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

function formatCurrency(value: number | string | null | undefined) {
  const amount = Number(value ?? 0)

  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2,
  }).format(Number.isFinite(amount) ? amount : 0)
}

function formatLocation(party: Party) {
  return [party.city, party.state].filter(Boolean).join(', ') || '—'
}

function showSuccess(message: string) {
  if (messageTimer) clearTimeout(messageTimer)

  successMessage.value = message
  messageTimer = setTimeout(() => {
    successMessage.value = ''
  }, 4000)
}

async function loadCustomers() {
  const sequence = ++requestSequence
  loading.value = true
  pageError.value = ''

  try {
    const response = await getParties({
      search: search.value.trim() || undefined,
      role: 'CUSTOMER',
      page: page.value,
      limit: pageSize,
      active:
        activeFilter.value === 'all'
          ? undefined
          : activeFilter.value === 'active',
    })

    if (sequence !== requestSequence) return

    customers.value = response.data ?? []
    total.value = Number(response.total ?? 0)

    const lastPage = Math.max(1, Math.ceil(total.value / pageSize))

    if (page.value > lastPage) {
      page.value = lastPage
      return
    }
  } catch (error: unknown) {
    if (sequence !== requestSequence) return

    customers.value = []
    total.value = 0

    pageError.value =
      error instanceof Error
        ? error.message
        : 'Unable to load customers. Please try again.'
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
      void loadCustomers()
    }
  }, 300)
}

function resetFilters() {
  if (searchTimer) clearTimeout(searchTimer)

  search.value = ''
  activeFilter.value = 'active'
  page.value = 1
  void loadCustomers()
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

  showSuccess('Customer saved successfully.')
  await loadCustomers()
}

async function handleDeactivate(party: Party) {
  if (!party.is_active || actionLoading.value) return

  const confirmed = window.confirm(
    `Deactivate "${party.name}"?\n\nThe customer record will be retained and can be viewed using the inactive filter.`,
  )

  if (!confirmed) return

  actionLoading.value = party.id
  pageError.value = ''
  successMessage.value = ''

  try {
    await deactivateParty(party.id)

    showSuccess('Customer deactivated successfully.')

    if (customers.value.length === 1 && page.value > 1) {
      page.value -= 1
    } else {
      await loadCustomers()
    }
  } catch (error: unknown) {
    pageError.value =
      error instanceof Error
        ? error.message
        : 'Unable to deactivate this customer.'
  } finally {
    actionLoading.value = null
  }
}

function changePage(nextPage: number) {
  if (nextPage < 1 || nextPage > totalPages.value || loading.value) return

  page.value = nextPage
}

watch([activeFilter, page], () => {
  void loadCustomers()
})

onMounted(() => {
  void loadCustomers()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  if (messageTimer) clearTimeout(messageTimer)

  requestSequence++
})
</script>

<template>
  <main class="customers-page">
    <!-- Page heading -->
    <header class="page-heading">
      <div class="heading-copy">
        <div class="section-eyebrow">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <circle cx="9" cy="8" r="3.2" />
            <path d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3" />
            <path d="M16 5.5a3 3 0 0 1 0 5.8M18 14c2 .7 3 2.4 3 5" />
          </svg>
          <span>CUSTOMER MANAGEMENT</span>
        </div>

        <h1>Customers</h1>
        <p>Manage customer records, contact details and credit information.</p>
      </div>

      <div class="heading-actions">
        <button
          type="button"
          class="secondary-button"
          :disabled="loading"
          @click="loadCustomers"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <path d="M20 7v5h-5" />
            <path d="M4 17v-5h5" />
            <path d="M5.6 9A7 7 0 0 1 18 6l2 6" />
            <path d="M18.4 15A7 7 0 0 1 6 18l-2-6" />
          </svg>
          Refresh
        </button>

        <button
          type="button"
          class="primary-button"
          @click="openCreateModal"
        >
          <span class="button-icon">+</span>
          Add customer
        </button>
      </div>
    </header>

    <!-- Feedback -->
    <div
      v-if="successMessage"
      class="notice notice-success"
      role="status"
    >
      <span>{{ successMessage }}</span>
      <button
        type="button"
        aria-label="Dismiss notification"
        @click="successMessage = ''"
      >
        ×
      </button>
    </div>

    <div
      v-if="pageError"
      class="notice notice-error"
      role="alert"
    >
      <span>{{ pageError }}</span>
      <button
        type="button"
        class="notice-retry"
        @click="loadCustomers"
      >
        Try again
      </button>
    </div>

    <!-- Summary -->
    <section class="summary-grid" aria-label="Customer summary">
      <article class="summary-card">
        <span class="summary-icon icon-neutral">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <circle cx="9" cy="8" r="3.2" />
            <path d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3" />
            <path d="M17 8h4M19 6v4" />
          </svg>
        </span>
        <div class="summary-content">
          <span class="summary-label">Matching customers</span>
          <strong>{{ loading ? '—' : total.toLocaleString('en-IN') }}</strong>
        </div>
      </article>

      <article class="summary-card">
        <span class="summary-icon icon-neutral">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <rect x="4" y="4" width="16" height="16" rx="3" />
            <path d="M8 9h8M8 13h8M8 17h4" />
          </svg>
        </span>
        <div class="summary-content">
          <span class="summary-label">Current page</span>
          <strong>{{ loading ? '—' : `${page} / ${totalPages}` }}</strong>
        </div>
      </article>

      <article class="summary-card">
        <span class="summary-icon icon-neutral">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <path d="M4 5h16v14H4z" />
            <path d="M8 9h8M8 13h8" />
          </svg>
        </span>
        <div class="summary-content">
          <span class="summary-label">Records displayed</span>
          <strong>{{ loading ? '—' : customers.length }}</strong>
        </div>
      </article>
    </section>

    <!-- Customer directory -->
    <section class="directory-card">
      <div class="directory-titlebar">
        <div>
          <h2>Customer register</h2>
          <p>Search, review and manage individual customer records.</p>
        </div>

        <span class="record-count">
          {{ total.toLocaleString('en-IN') }} records
        </span>
      </div>

      <!-- Search and filters -->
      <div class="directory-toolbar">
        <label class="search-field">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <circle cx="11" cy="11" r="7" />
            <path d="m16 16 4 4" />
          </svg>

          <span class="sr-only">Search customers</span>

          <input
            v-model="search"
            type="search"
            placeholder="Search name, code, phone or GSTIN..."
            @input="handleSearch"
          >
        </label>

        <label class="select-field">
          <span class="sr-only">Filter customer status</span>
          <select v-model="activeFilter">
            <option value="active">Active customers</option>
            <option value="inactive">Inactive customers</option>
            <option value="all">All customers</option>
          </select>
        </label>

        <button
          type="button"
          class="reset-button"
          :disabled="loading && !search && activeFilter === 'active'"
          @click="resetFilters"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <path d="M3 12a9 9 0 1 0 2.6-6.4L3 8" />
            <path d="M3 3v5h5" />
          </svg>
          Reset
        </button>
      </div>

      <!-- Table -->
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th>Customer</th>
              <th>Contact</th>
              <th>GSTIN</th>
              <th>Location</th>
              <th class="numeric-cell">Opening balance</th>
              <th>Status</th>
              <th class="actions-heading">Actions</th>
            </tr>
          </thead>

          <tbody>
            <tr v-if="loading && customers.length === 0">
              <td colspan="7" class="state-cell">
                <span class="loading-indicator" />
                Loading customers...
              </td>
            </tr>

            <tr v-else-if="!pageError && customers.length === 0">
              <td colspan="7" class="state-cell">
                <div class="empty-state">
                  <span class="empty-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.7"
                      aria-hidden="true"
                    >
                      <circle cx="9" cy="8" r="3.2" />
                      <path d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3" />
                      <path d="M17 8h4M19 6v4" />
                    </svg>
                  </span>

                  <strong>
                    {{ search.trim() ? 'No matching customers' : 'No customers found' }}
                  </strong>

                  <p>
                    {{
                      search.trim()
                        ? 'Try a different search term or change the status filter.'
                        : 'Add a customer to start building your directory.'
                    }}
                  </p>

                  <button
                    v-if="!search.trim() && activeFilter !== 'inactive'"
                    type="button"
                    class="secondary-button"
                    @click="openCreateModal"
                  >
                    Add your first customer
                  </button>
                </div>
              </td>
            </tr>

            <template v-else>
              <tr
                v-for="customer in customers"
                :key="customer.id"
                class="data-row"
              >
                <td>
                  <div class="party-cell">
                    <span class="party-avatar">
                      {{ customer.name.trim().charAt(0).toUpperCase() || 'C' }}
                    </span>

                    <div class="party-identity">
                      <button
                        type="button"
                        class="party-name"
                        @click="openEditModal(customer)"
                      >
                        {{ customer.name }}
                      </button>
                      <span>{{ customer.party_code || 'No code' }}</span>
                    </div>
                  </div>
                </td>

                <td>
                  <div class="contact-cell">
                    <span>{{ customer.phone || '—' }}</span>
                    <small>
                      {{ customer.email || customer.contact_person || '—' }}
                    </small>
                  </div>
                </td>

                <td>
                  <span class="muted-text">{{ customer.gstin || '—' }}</span>
                </td>

                <td>
                  <div class="location-cell">
                    <span>{{ formatLocation(customer) }}</span>
                    <small v-if="customer.pincode">
                      {{ customer.pincode }}
                    </small>
                  </div>
                </td>

                <td class="numeric-cell">
                  <div class="balance-cell">
                    <strong>{{ formatCurrency(customer.opening_balance) }}</strong>
                    <small>
                      {{ customer.opening_balance_type || 'Opening balance' }}
                    </small>
                  </div>
                </td>

                <td>
                  <span
                    class="status-badge"
                    :class="customer.is_active ? 'status-active' : 'status-inactive'"
                  >
                    <span class="status-dot" />
                    {{ customer.is_active ? 'Active' : 'Inactive' }}
                  </span>
                </td>

                <td>
                  <div class="row-actions">
                    <button
                      type="button"
                      class="row-action"
                      :aria-label="`Edit ${customer.name}`"
                      @click="openEditModal(customer)"
                    >
                      Edit
                    </button>

                    <button
                      v-if="customer.is_active"
                      type="button"
                      class="row-action row-action-danger"
                      :disabled="actionLoading === customer.id"
                      :aria-label="`Deactivate ${customer.name}`"
                      @click="handleDeactivate(customer)"
                    >
                      {{
                        actionLoading === customer.id
                          ? 'Saving...'
                          : 'Deactivate'
                      }}
                    </button>
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      <footer class="directory-footer">
        <span>
          Showing
          <strong>{{ startRecord }}–{{ endRecord }}</strong>
          of
          <strong>{{ total.toLocaleString('en-IN') }}</strong>
          customers
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

          <span class="page-indicator">
            {{ page }} / {{ totalPages }}
          </span>

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
  default-role="CUSTOMER"
  @close="modalOpen = false"
  @saved="handleSaved"
/>
  </main>
</template>

<style scoped>
.customers-page {
  width: 100%;
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
  letter-spacing: 0.09em;
}

.section-eyebrow svg {
  width: 16px;
  height: 16px;
  flex: 0 0 auto;
}

.page-heading h1 {
  margin: 0;
  font-size: 27px;
  font-weight: 700;
  line-height: 1.25;
  letter-spacing: -0.045em;
}

.page-heading p {
  margin: 6px 0 0;
  color: #737373;
  font-size: 13px;
  line-height: 1.5;
}

.heading-actions {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 9px;
}

.primary-button,
.secondary-button,
.reset-button,
.pagination-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 38px;
  border-radius: 7px;
  padding: 0 13px;
  font-size: 12px;
  font-weight: 550;
  white-space: nowrap;
  transition: background-color 150ms ease, border-color 150ms ease;
}

.primary-button {
  border: 1px solid #171717;
  background: #171717;
  color: #fff;
}

.primary-button:hover {
  background: #333;
}

.primary-button:disabled,
.secondary-button:disabled,
.reset-button:disabled,
.pagination-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.button-icon {
  font-size: 19px;
  line-height: 1;
}

.secondary-button,
.reset-button,
.pagination-button {
  border: 1px solid #d4d4d4;
  background: #fff;
  color: #262626;
}

.secondary-button:hover,
.reset-button:hover:not(:disabled),
.pagination-button:hover:not(:disabled) {
  background: #f5f5f5;
  border-color: #a3a3a3;
}

.secondary-button svg,
.reset-button svg,
.primary-button svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.notice {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  margin-bottom: 14px;
  border: 1px solid;
  border-radius: 7px;
  padding: 11px 14px;
  font-size: 12px;
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

.notice > button {
  flex: 0 0 auto;
  font-size: 19px;
  line-height: 1;
}

.notice-retry {
  font-size: 12px !important;
  font-weight: 600;
  text-decoration: underline;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 14px;
}

.summary-card {
  display: flex;
  min-height: 76px;
  align-items: center;
  gap: 12px;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  background: #fff;
  padding: 13px 15px;
}

.summary-icon {
  display: inline-flex;
  width: 36px;
  height: 36px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 1px solid #e5e5e5;
  border-radius: 7px;
  background: #f5f5f5;
  color: #525252;
}

.summary-icon svg {
  width: 18px;
  height: 18px;
}

.summary-content {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
}

.summary-label {
  color: #737373;
  font-size: 12px;
}

.summary-content strong {
  color: #171717;
  font-size: 20px;
  font-weight: 700;
  line-height: 1.1;
  font-variant-numeric: tabular-nums;
}

.directory-card {
  overflow: hidden;
  border: 1px solid #e5e5e5;
  border-radius: 9px;
  background: #fff;
}

.directory-titlebar {
  display: flex;
  min-height: 66px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 13px 16px;
}

.directory-titlebar h2 {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
  letter-spacing: -0.015em;
}

.directory-titlebar p {
  margin: 5px 0 0;
  color: #737373;
  font-size: 12px;
}

.record-count {
  flex: 0 0 auto;
  border: 1px solid #e5e5e5;
  border-radius: 5px;
  padding: 5px 8px;
  color: #525252;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.directory-toolbar {
  display: grid;
  grid-template-columns: minmax(240px, 1fr) minmax(160px, 190px) auto;
  align-items: center;
  gap: 9px;
  border-top: 1px solid #ededed;
  border-bottom: 1px solid #e5e5e5;
  background: #fff;
  padding: 12px 14px;
}

.search-field {
  display: flex;
  min-width: 0;
  height: 39px;
  align-items: center;
  gap: 9px;
  border: 1px solid #d4d4d4;
  border-radius: 6px;
  background: #fff;
  padding: 0 11px;
}

.search-field:focus-within {
  border-color: #737373;
  box-shadow: 0 0 0 2px rgb(23 23 23 / 5%);
}

.search-field svg {
  width: 16px;
  height: 16px;
  flex: 0 0 auto;
  color: #737373;
}

.search-field input {
  width: 100%;
  min-width: 0;
  height: 100%;
  border: 0;
  outline: none;
  background: transparent;
  color: #171717;
  font-size: 12px;
}

.search-field input::placeholder {
  color: #a3a3a3;
}

.select-field {
  display: block;
  min-width: 0;
}

.select-field select {
  width: 100%;
  height: 39px;
  border: 1px solid #d4d4d4;
  border-radius: 6px;
  background: #fff;
  padding: 0 10px;
  color: #262626;
  font-size: 12px;
  outline: none;
}

.select-field select:focus {
  border-color: #737373;
}

.reset-button {
  min-height: 39px;
  padding: 0 12px;
}

.table-scroll {
  width: 100%;
  overflow-x: auto;
}

table {
  width: 100%;
  min-width: 950px;
  border-collapse: collapse;
  text-align: left;
}

thead {
  background: #fafafa;
}

thead tr {
  border-bottom: 1px solid #e5e5e5;
}

th {
  height: 38px;
  padding: 0 14px;
  color: #737373;
  font-size: 10px;
  font-weight: 650;
  letter-spacing: 0.055em;
  text-transform: uppercase;
  white-space: nowrap;
}

td {
  height: 62px;
  border-bottom: 1px solid #ededed;
  padding: 10px 14px;
  color: #404040;
  font-size: 12px;
  vertical-align: middle;
}

.data-row {
  transition: background-color 120ms ease;
}

.data-row:hover {
  background: #fafafa;
}

tbody tr:last-child td {
  border-bottom: 0;
}

.party-cell {
  display: flex;
  min-width: 165px;
  align-items: center;
  gap: 10px;
}

.party-avatar {
  display: inline-flex;
  width: 35px;
  height: 35px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 1px solid #e5e5e5;
  border-radius: 7px;
  background: #f5f5f5;
  color: #404040;
  font-size: 14px;
  font-weight: 650;
  text-transform: uppercase;
}

.party-identity,
.contact-cell,
.location-cell,
.balance-cell {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
}

.party-name {
  width: fit-content;
  max-width: 220px;
  overflow: hidden;
  color: #171717;
  font-size: 12px;
  font-weight: 650;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.party-name:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.party-identity > span,
.contact-cell small,
.location-cell small,
.balance-cell small {
  color: #737373;
  font-size: 11px;
  line-height: 1.35;
}

.contact-cell > span,
.location-cell > span {
  overflow: hidden;
  color: #262626;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-cell small {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.muted-text {
  color: #525252;
  font-size: 12px;
  white-space: nowrap;
}

.numeric-cell {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.balance-cell {
  align-items: flex-end;
}

.balance-cell strong {
  color: #171717;
  font-size: 12px;
  font-weight: 650;
  white-space: nowrap;
}

.balance-cell small {
  white-space: nowrap;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  border: 1px solid #a7f3d0;
  border-radius: 999px;
  background: #ecfdf5;
  padding: 5px 9px;
  color: #047857;
  font-size: 11px;
  font-weight: 550;
  white-space: nowrap;
}

.status-dot {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: #059669;
}

.status-inactive {
  border-color: #e5e5e5;
  background: #f5f5f5;
  color: #737373;
}

.status-inactive .status-dot {
  background: #a3a3a3;
}

.actions-heading {
  text-align: right;
}

.row-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 13px;
  white-space: nowrap;
}

.row-action {
  color: #404040;
  font-size: 11px;
  font-weight: 600;
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
  height: 210px;
  padding: 25px 16px;
  color: #737373;
  text-align: center;
}

.loading-indicator {
  display: inline-block;
  width: 17px;
  height: 17px;
  margin-right: 9px;
  border: 2px solid #d4d4d4;
  border-top-color: #171717;
  border-radius: 50%;
  vertical-align: middle;
  animation: spin 0.7s linear infinite;
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
  gap: 9px;
}

.empty-icon {
  display: inline-flex;
  width: 43px;
  height: 43px;
  align-items: center;
  justify-content: center;
  border: 1px solid #e5e5e5;
  border-radius: 9px;
  background: #fafafa;
  color: #737373;
}

.empty-icon svg {
  width: 21px;
  height: 21px;
}

.empty-state strong {
  color: #262626;
  font-size: 13px;
  font-weight: 650;
}

.empty-state p {
  margin: 0 0 3px;
  color: #737373;
  font-size: 12px;
}

.directory-footer {
  display: flex;
  min-height: 57px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  border-top: 1px solid #e5e5e5;
  padding: 10px 15px;
  color: #737373;
  font-size: 12px;
}

.directory-footer > span strong {
  color: #404040;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.pagination {
  display: flex;
  align-items: center;
  gap: 9px;
}

.pagination-button {
  min-height: 33px;
  padding: 0 11px;
  font-size: 11px;
}

.page-indicator {
  min-width: 49px;
  color: #525252;
  text-align: center;
  font-size: 12px;
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
  .customers-page {
    padding-right: 30px;
    padding-left: 30px;
  }

  td {
    height: 64px;
  }
}

@media (max-width: 900px) {
  .customers-page {
    padding: 18px;
  }

  .directory-toolbar {
    grid-template-columns: minmax(200px, 1fr) minmax(145px, 180px) auto;
  }

  .summary-card {
    padding: 12px;
  }
}

@media (max-width: 640px) {
  .customers-page {
    padding: 15px 12px 22px;
  }

  .page-heading {
    align-items: flex-start;
    flex-direction: column;
    gap: 14px;
  }

  .heading-actions {
    width: 100%;
  }

  .heading-actions button {
    flex: 1;
  }

  .page-heading h1 {
    font-size: 25px;
  }

  .summary-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .summary-card {
    min-height: 65px;
  }

  .directory-titlebar {
    align-items: flex-start;
  }

  .directory-toolbar {
    grid-template-columns: minmax(0, 1fr) minmax(130px, 0.7fr);
  }

  .search-field {
    grid-column: 1 / -1;
  }

  .reset-button {
    width: 100%;
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
</style>
