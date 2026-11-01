<script setup lang="ts">
import {
  computed,
  onMounted,
  onBeforeUnmount,
  ref,
  watch,
} from 'vue'
import type { Party, PartyRole } from '~/composables/useApi'
import PartyFormModal from '~/components/parties/PartyFormModal.vue'
type RoleFilter = 'ALL' | PartyRole
type StatusFilter = 'ACTIVE' | 'INACTIVE' | 'ALL'

const { getParties, deactivateParty } = useApi()

const parties = ref<Party[]>([])
const search = ref('')
const roleFilter = ref<RoleFilter>('ALL')
const statusFilter = ref<StatusFilter>('ACTIVE')

const page = ref(1)
const pageSize = 25
const total = ref(0)

const loading = ref(false)
const actionLoading = ref<string | null>(null)

const errorMessage = ref('')
const successMessage = ref('')

const modalOpen = ref(false)
const editingParty = ref<Party | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | undefined
let messageTimer: ReturnType<typeof setTimeout> | undefined
let requestSequence = 0

const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / pageSize)),
)

const rangeStart = computed(() =>
  total.value === 0 ? 0 : (page.value - 1) * pageSize + 1,
)

const rangeEnd = computed(() =>
  Math.min(page.value * pageSize, total.value),
)

const supplierCount = computed(() =>
  parties.value.filter(party =>
    party.roles?.includes('SUPPLIER'),
  ).length,
)

const customerCount = computed(() =>
  parties.value.filter(party =>
    party.roles?.includes('CUSTOMER'),
  ).length,
)

const formatCurrency = (
  value: number | string | null | undefined,
) => {
  const amount = Number(value ?? 0)

  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2,
  }).format(Number.isFinite(amount) ? amount : 0)
}

const formatRoles = (roles: PartyRole[] = []) => {
  if (
    roles.includes('SUPPLIER') &&
    roles.includes('CUSTOMER')
  ) {
    return 'Supplier · Customer'
  }

  if (roles.includes('SUPPLIER')) return 'Supplier'
  if (roles.includes('CUSTOMER')) return 'Customer'

  return 'Unassigned'
}

const balanceLabel = (party: Party) => {
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

const showMessage = (
  message: string,
  type: 'success' | 'error',
) => {
  if (messageTimer) clearTimeout(messageTimer)

  if (type === 'success') {
    successMessage.value = message
    errorMessage.value = ''
  } else {
    errorMessage.value = message
    successMessage.value = ''
  }

  messageTimer = setTimeout(() => {
    successMessage.value = ''
    errorMessage.value = ''
  }, 5000)
}

const getErrorMessage = (
  error: unknown,
  fallback: string,
) => {
  if (
    error &&
    typeof error === 'object' &&
    'data' in error
  ) {
    const data = (
      error as {
        data?: {
          message?: string
          error?: string
        }
      }
    ).data

    return data?.message ?? data?.error ?? fallback
  }

  return error instanceof Error
    ? error.message
    : fallback
}

const loadParties = async () => {
  const sequence = ++requestSequence

  loading.value = true
  errorMessage.value = ''

  try {
    const response = await getParties({
      search: search.value.trim() || undefined,

      role:
        roleFilter.value === 'ALL'
          ? undefined
          : roleFilter.value,

      page: page.value,
      limit: pageSize,

      active:
        statusFilter.value === 'ALL'
          ? undefined
          : statusFilter.value === 'ACTIVE',
    })

    if (sequence !== requestSequence) return

    parties.value = response.data ?? []
    total.value = Number(response.total ?? 0)

    const lastPage = Math.max(
      1,
      Math.ceil(total.value / pageSize),
    )

    if (page.value > lastPage) {
      page.value = lastPage
    }
  } catch (error: unknown) {
    if (sequence !== requestSequence) return

    parties.value = []
    total.value = 0

    errorMessage.value = getErrorMessage(
      error,
      'Unable to load parties.',
    )
  } finally {
    if (sequence === requestSequence) {
      loading.value = false
    }
  }
}

const scheduleSearch = () => {
  if (searchTimer) clearTimeout(searchTimer)

  searchTimer = setTimeout(() => {
    page.value = 1
    void loadParties()
  }, 300)
}

const resetFilters = () => {
  if (searchTimer) clearTimeout(searchTimer)

  search.value = ''
  roleFilter.value = 'ALL'
  statusFilter.value = 'ACTIVE'
  page.value = 1

  void loadParties()
}

const openCreateModal = () => {
  editingParty.value = null
  modalOpen.value = true
}

const openEditModal = (party: Party) => {
  editingParty.value = party
  modalOpen.value = true
}

const handleSaved = async () => {
  modalOpen.value = false
  editingParty.value = null

  showMessage('Party saved successfully.', 'success')

  await loadParties()
}

const handleDeactivate = async (party: Party) => {
  const confirmed = window.confirm(
    `Deactivate "${party.name}"?\n\nThe party will no longer appear in the active directory.`,
  )

  if (!confirmed) return

  actionLoading.value = party.id
  errorMessage.value = ''

  try {
    await deactivateParty(party.id)

    showMessage(
      'Party deactivated successfully.',
      'success',
    )

    await loadParties()
  } catch (error: unknown) {
    showMessage(
      getErrorMessage(
        error,
        'Unable to deactivate this party.',
      ),
      'error',
    )
  } finally {
    actionLoading.value = null
  }
}

const changePage = (nextPage: number) => {
  if (
    nextPage < 1 ||
    nextPage > totalPages.value ||
    loading.value
  ) {
    return
  }

  page.value = nextPage
  void loadParties()
}

watch(
  [roleFilter, statusFilter],
  () => {
    page.value = 1
    void loadParties()
  },
)

onMounted(() => {
  void loadParties()
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
  if (messageTimer) clearTimeout(messageTimer)
})
</script>

<template>
  <main class="parties-page">
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
            <path
              d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3"
            />
            <path
              d="M16 5.5a3 3 0 0 1 0 5.8M18 14c2 .7 3 2.4 3 5"
            />
          </svg>

          <span>PARTY MANAGEMENT</span>
        </div>

        <h1>Parties</h1>

        <p>
          Manage suppliers, customers and their business information.
        </p>
      </div>

      <div class="heading-actions">
        <button
          class="secondary-button"
          type="button"
          :disabled="loading"
          @click="loadParties"
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
          class="primary-button"
          type="button"
          @click="openCreateModal"
        >
          <span class="button-icon">+</span>
          Add party
        </button>
      </div>
    </header>

    <!-- Notifications -->
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
      v-if="errorMessage"
      class="notice notice-error"
      role="alert"
    >
      <span>{{ errorMessage }}</span>

      <button
        type="button"
        aria-label="Dismiss error"
        @click="errorMessage = ''"
      >
        ×
      </button>
    </div>

    <!-- Summary cards -->
    <section
      class="summary-grid"
      aria-label="Party summary"
    >
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
            <path
              d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3M17 8h4M19 6v4"
            />
          </svg>
        </span>

        <div class="summary-content">
          <span class="summary-label">Matching parties</span>
          <strong>{{ total.toLocaleString('en-IN') }}</strong>
        </div>
      </article>

      <article class="summary-card">
        <span class="summary-icon icon-success">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
            <path d="m5 12 4 4L19 6" />
          </svg>
        </span>

        <div class="summary-content">
          <span class="summary-label">Suppliers on page</span>
          <strong>{{ supplierCount.toLocaleString('en-IN') }}</strong>
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
            <circle cx="9" cy="8" r="3.2" />
            <path
              d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3M16 6h5M16 10h5"
            />
          </svg>
        </span>

        <div class="summary-content">
          <span class="summary-label">Customers on page</span>
          <strong>{{ customerCount.toLocaleString('en-IN') }}</strong>
        </div>
      </article>
    </section>

    <!-- Party register -->
    <section class="directory-card">
      <div class="directory-titlebar">
        <div>
          <h2>Party register</h2>
          <p>
            Search, review and manage individual parties.
          </p>
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

          <span class="sr-only">Search parties</span>

          <input
            v-model="search"
            type="search"
            placeholder="Search name, party code, phone or GSTIN..."
            @input="scheduleSearch"
          />
        </label>

        <label class="select-field">
          <span class="sr-only">Filter by role</span>

          <select v-model="roleFilter">
            <option value="ALL">All roles</option>
            <option value="SUPPLIER">Suppliers</option>
            <option value="CUSTOMER">Customers</option>
          </select>
        </label>

        <label class="select-field">
          <span class="sr-only">Filter by status</span>

          <select v-model="statusFilter">
            <option value="ACTIVE">Active parties</option>
            <option value="INACTIVE">Inactive parties</option>
            <option value="ALL">All statuses</option>
          </select>
        </label>

        <button
          class="reset-button"
          type="button"
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
              <th>Party</th>
              <th>Role</th>
              <th>Contact</th>
              <th>GSTIN</th>
              <th>Location</th>
              <th class="numeric-cell">Opening balance</th>
              <th>Status</th>
              <th class="actions-heading">Actions</th>
            </tr>
          </thead>

          <tbody>
            <tr v-if="loading && parties.length === 0">
              <td colspan="8" class="state-cell">
                <span class="loading-indicator"></span>
                Loading parties...
              </td>
            </tr>

            <tr v-else-if="parties.length === 0">
              <td colspan="8" class="state-cell">
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
                      <path
                        d="M3 19c0-3.1 2.5-5.3 6-5.3s6 2.2 6 5.3M17 8h4M19 6v4"
                      />
                    </svg>
                  </span>

                  <strong>No parties found</strong>

                  <p>
                    {{
                      search
                        ? 'Try a different search term or adjust your filters.'
                        : 'Add a party to start building your directory.'
                    }}
                  </p>

                  <button
                    v-if="!search"
                    type="button"
                    class="secondary-button"
                    @click="openCreateModal"
                  >
                    Add your first party
                  </button>
                </div>
              </td>
            </tr>

            <tr
              v-for="party in parties"
              v-else
              :key="party.id"
            >
              <td>
                <div class="party-cell">
                  <span class="party-avatar">
                    {{ party.name.trim().charAt(0).toUpperCase() || 'P' }}
                  </span>

                  <div class="party-identity">
                    <button
                      type="button"
                      class="party-name"
                      @click="openEditModal(party)"
                    >
                      {{ party.name }}
                    </button>

                    <span>{{ party.party_code || '—' }}</span>
                  </div>
                </div>
              </td>

              <td>
                <span class="role-label">
                  {{ formatRoles(party.roles) }}
                </span>
              </td>

              <td>
                <div class="contact-cell">
                  <span>{{ party.phone || '—' }}</span>

                  <small v-if="party.contact_person">
                    {{ party.contact_person }}
                  </small>
                </div>
              </td>

              <td>
                <span class="muted-text">
                  {{ party.gstin || '—' }}
                </span>
              </td>

              <td>
                <div class="location-cell">
                  <span>{{ party.city || '—' }}</span>

                  <small v-if="party.state">
                    {{ party.state }}
                  </small>
                </div>
              </td>

              <td class="numeric-cell">
                <div class="balance-cell">
                  <strong>
                    {{ formatCurrency(party.opening_balance) }}
                  </strong>

                  <small>{{ balanceLabel(party) }}</small>
                </div>
              </td>

              <td>
                <span
                  class="status-badge"
                  :class="
                    party.is_active
                      ? 'status-active'
                      : 'status-inactive'
                  "
                >
                  <span class="status-dot"></span>
                  {{ party.is_active ? 'Active' : 'Inactive' }}
                </span>
              </td>

              <td>
                <div class="row-actions">
                  <button
                    type="button"
                    class="row-action"
                    title="Edit party"
                    :aria-label="`Edit ${party.name}`"
                    @click="openEditModal(party)"
                  >
                    Edit
                  </button>

                  <button
                    v-if="party.is_active"
                    type="button"
                    class="row-action row-action-danger"
                    :disabled="actionLoading === party.id"
                    :aria-label="`Deactivate ${party.name}`"
                    @click="handleDeactivate(party)"
                  >
                    {{
                      actionLoading === party.id
                        ? 'Saving...'
                        : 'Deactivate'
                    }}
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      <footer class="directory-footer">
        <span>
          Showing
          <strong>{{ rangeStart }}–{{ rangeEnd }}</strong>
          of
          <strong>{{ total.toLocaleString('en-IN') }}</strong>
          parties
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

    <!-- Create / edit modal -->
    <PartyFormModal
      :open="modalOpen"
      :party="editingParty"
      @close="modalOpen = false"
      @saved="handleSaved"
    />
  </main>
</template>

<style scoped>
/* Page */
.parties-page {
  width: 100%;
  padding: 18px 14px 26px;
  color: #111827;
  font-size: 13px;
}

/* Heading */
.page-heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 16px;
}

.section-eyebrow {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
  color: #657184;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.1px;
}

.section-eyebrow svg {
  width: 14px;
  height: 14px;
}

.page-heading h1 {
  margin: 0;
  color: #111827;
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.65px;
  line-height: 1.2;
}

.page-heading p,
.directory-titlebar p {
  margin: 5px 0 0;
  color: #788397;
  font-size: 12px;
  line-height: 1.5;
}

.heading-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Buttons */
button,
input,
select {
  font: inherit;
}

button {
  cursor: pointer;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.primary-button,
.secondary-button,
.pagination-button,
.reset-button {
  display: inline-flex;
  min-height: 34px;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 0 12px;
  border: 1px solid #dfe3e9;
  border-radius: 5px;
  background: #fff;
  color: #344054;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  transition:
    background 0.15s ease,
    border-color 0.15s ease;
}

.primary-button {
  border-color: #17191d;
  background: #17191d;
  color: #fff;
}

.primary-button:hover {
  background: #30343a;
}

.secondary-button:hover:not(:disabled),
.pagination-button:hover:not(:disabled),
.reset-button:hover {
  border-color: #b9c1cc;
  background: #f8f9fb;
}

.primary-button svg,
.secondary-button svg,
.reset-button svg {
  width: 14px;
  height: 14px;
}

.button-icon {
  font-size: 18px;
  line-height: 1;
}

/* Summary cards */
.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
  margin-bottom: 14px;
}

.summary-card {
  display: flex;
  min-height: 66px;
  align-items: center;
  gap: 11px;
  padding: 10px 12px;
  border: 1px solid #e0e4ea;
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
  border-radius: 6px;
  background: #f1f3f5;
  color: #4b5563;
}

.summary-icon svg {
  width: 17px;
  height: 17px;
}

.icon-success {
  background: #e8f7ef;
  color: #059669;
}

.summary-content {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.summary-label {
  color: #657184;
  font-size: 11px;
}

.summary-card strong {
  color: #111827;
  font-size: 18px;
  font-weight: 700;
  line-height: 1.15;
}

/* Register panel */
.directory-card {
  overflow: hidden;
  border: 1px solid #dfe3e9;
  border-radius: 8px;
  background: #fff;
}

.directory-titlebar {
  display: flex;
  min-height: 58px;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 11px 13px;
  border-bottom: 1px solid #e7eaef;
}

.directory-titlebar h2 {
  margin: 0;
  color: #111827;
  font-size: 13px;
  font-weight: 700;
}

.record-count {
  padding: 5px 8px;
  border: 1px solid #e3e7ed;
  border-radius: 4px;
  color: #657184;
  font-size: 10px;
  white-space: nowrap;
}

/* Search and filters */
.directory-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 10px;
  border-bottom: 1px solid #e7eaef;
}

.search-field {
  display: flex;
  min-width: 180px;
  min-height: 36px;
  flex: 1 1 auto;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  border: 1px solid #d9dfe7;
  border-radius: 5px;
  background: #fff;
}

.search-field:focus-within {
  border-color: #9aa4b2;
  box-shadow: 0 0 0 2px rgb(148 163 184 / 10%);
}

.search-field svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
  color: #7d8797;
}

.search-field input {
  width: 100%;
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: #111827;
  font-size: 12px;
}

.search-field input::placeholder {
  color: #8b95a5;
}

.select-field select {
  width: 125px;
  min-height: 36px;
  padding: 0 9px;
  border: 1px solid #d9dfe7;
  border-radius: 5px;
  outline: 0;
  background: #fff;
  color: #344054;
  font-size: 11px;
}

.select-field select:focus {
  border-color: #9aa4b2;
}

.reset-button {
  border-color: transparent;
  color: #697586;
  font-weight: 500;
}

/* Table */
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
  background: #fafbfc;
}

th {
  padding: 11px 12px;
  border-bottom: 1px solid #e1e6ed;
  color: #667085;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.55px;
  text-transform: uppercase;
}

td {
  padding: 11px 12px;
  border-bottom: 1px solid #e9edf2;
  color: #253044;
  font-size: 12px;
  vertical-align: middle;
}

tbody tr:last-child td {
  border-bottom: 0;
}

tbody tr:hover:not(:has(.state-cell)) {
  background: #fafbfc;
}

/* Party name and contact */
.party-cell {
  display: flex;
  align-items: center;
  gap: 9px;
}

.party-avatar {
  display: inline-flex;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 1px solid #e1e5eb;
  border-radius: 6px;
  background: #f5f6f8;
  color: #4b5563;
  font-size: 12px;
  font-weight: 700;
}

.party-identity,
.contact-cell,
.location-cell,
.balance-cell {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.party-name {
  max-width: 240px;
  overflow: hidden;
  padding: 0;
  border: 0;
  background: transparent;
  color: #182230;
  font-size: 12px;
  font-weight: 700;
  text-align: left;
  text-overflow: ellipsis;
}

.party-name:hover {
  text-decoration: underline;
}

.party-identity > span,
.contact-cell small,
.location-cell small,
.balance-cell small {
  color: #8993a3;
  font-size: 10px;
}

.role-label,
.muted-text {
  color: #5d6879;
  font-size: 11px;
}

/* Balance */
.numeric-cell {
  text-align: right;
}

.numeric-cell .balance-cell {
  align-items: flex-end;
}

.balance-cell strong {
  color: #202939;
  font-size: 12px;
  font-weight: 600;
}

/* Status */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border: 1px solid #b8ecd1;
  border-radius: 999px;
  background: #ecfbf3;
  color: #087f50;
  font-size: 10px;
  font-weight: 600;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.status-inactive {
  border-color: #e0e4ea;
  background: #f4f5f7;
  color: #707987;
}

/* Row actions */
.actions-heading {
  text-align: right;
}

.row-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.row-action {
  padding: 5px 0;
  border: 0;
  background: transparent;
  color: #515d70;
  font-size: 11px;
  font-weight: 600;
}

.row-action:hover {
  color: #111827;
  text-decoration: underline;
}

.row-action-danger {
  color: #a43d3d;
}

/* Loading and empty states */
.state-cell {
  height: 175px;
  color: #7c8797;
  text-align: center;
}

.loading-indicator {
  display: inline-block;
  width: 15px;
  height: 15px;
  margin-right: 8px;
  border: 2px solid #d9dfe7;
  border-top-color: #4b5563;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  vertical-align: middle;
}

.empty-state {
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 9px;
  padding: 12px;
}

.empty-icon {
  display: flex;
  width: 38px;
  height: 38px;
  align-items: center;
  justify-content: center;
  border: 1px solid #e1e5eb;
  border-radius: 7px;
  background: #f6f7f9;
  color: #697586;
}

.empty-icon svg {
  width: 19px;
  height: 19px;
}

.empty-state strong {
  color: #273244;
  font-size: 13px;
}

.empty-state p {
  margin: 0 0 3px;
  color: #858e9e;
  font-size: 12px;
}

/* Pagination */
.directory-footer {
  display: flex;
  min-height: 48px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 12px;
  border-top: 1px solid #e7eaef;
  color: #7c8797;
  font-size: 11px;
}

.directory-footer strong {
  color: #465164;
  font-weight: 700;
}

.pagination {
  display: flex;
  align-items: center;
  gap: 10px;
}

.pagination-button {
  min-height: 31px;
  padding: 0 11px;
  font-size: 10px;
}

.page-indicator {
  min-width: 40px;
  color: #465164;
  text-align: center;
}

/* Notifications */
.notice {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  padding: 11px 13px;
  border: 1px solid;
  border-radius: 6px;
  font-size: 12px;
}

.notice button {
  border: 0;
  background: transparent;
  font-size: 18px;
}

.notice-success {
  border-color: #c9ead8;
  background: #f1fbf5;
  color: #287347;
}

.notice-error {
  border-color: #f0d5d5;
  background: #fff8f8;
  color: #a43d3d;
}

/* Accessibility */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  clip-path: inset(50%);
}

/* Animations */
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Responsive layout */
@media (max-width: 1100px) {
  .directory-toolbar {
    flex-wrap: wrap;
  }

  .search-field {
    flex-basis: 100%;
  }

  .select-field select {
    width: 145px;
  }
}

@media (max-width: 700px) {
  .parties-page {
    padding: 14px 10px 22px;
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
    justify-content: flex-end;
  }

  .summary-grid {
    grid-template-columns: 1fr;
    gap: 7px;
  }

  .summary-card {
    min-height: 60px;
  }

  .directory-titlebar {
    align-items: flex-start;
  }

  .directory-toolbar {
    align-items: stretch;
  }

  .search-field {
    flex-basis: 100%;
    min-height: 38px;
  }

  .select-field {
    flex: 1 1 0;
  }

  .select-field select {
    width: 100%;
    min-height: 38px;
  }

  .reset-button {
    min-height: 38px;
  }

  .directory-footer {
    align-items: flex-start;
    flex-direction: column;
    padding: 12px;
  }

  .pagination {
    width: 100%;
    justify-content: space-between;
  }
}
</style>