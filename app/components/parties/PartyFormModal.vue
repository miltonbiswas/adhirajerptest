<script setup lang="ts">
import { computed, reactive, watch, ref, onBeforeUnmount } from 'vue'
import type {
  Party,
  PartyInput,
  PartyRole,
  OpeningBalanceType,
} from '~/composables/useApi'

const props = withDefaults(
  defineProps<{
    open: boolean
    party?: Party | null
    defaultRole?: PartyRole
  }>(),
  {
    party: null,
    defaultRole: 'SUPPLIER',
  },
)

const emit = defineEmits<{
  close: []
  saved: [party: Party]
}>()

const { createParty, updateParty } = useApi()

const saving = ref(false)
const errorMessage = ref('')

const form = reactive({
  party_code: '',
  name: '',
  roles: ['SUPPLIER'] as PartyRole[],
  contact_person: '',
  phone: '',
  alternate_phone: '',
  email: '',
  gstin: '',
  pan: '',
  drug_license_number: '',
  billing_address: '',
  shipping_address: '',
  city: '',
  district: '',
  state: 'Rajasthan',
  pincode: '',
  country: 'India',
  opening_balance: 0,
  opening_balance_type: 'PAYABLE' as OpeningBalanceType,
  credit_limit: 0,
  credit_days: 0,
  notes: '',
  is_active: true,
})

const isEditing = computed(() => Boolean(props.party))

const title = computed(() => {
  if (isEditing.value) {
    return props.party?.roles?.includes('SUPPLIER') &&
      props.defaultRole === 'SUPPLIER'
      ? 'Edit supplier'
      : props.party?.roles?.includes('CUSTOMER') &&
          props.defaultRole === 'CUSTOMER'
        ? 'Edit customer'
        : 'Edit party'
  }

  return props.defaultRole === 'CUSTOMER'
    ? 'Add customer'
    : props.defaultRole === 'SUPPLIER'
      ? 'Add supplier'
      : 'Add party'
})

const description = computed(() => {
  if (isEditing.value) {
    return 'Update contact, business and account information.'
  }

  return props.defaultRole === 'CUSTOMER'
    ? 'Add a customer to your business directory.'
    : props.defaultRole === 'SUPPLIER'
      ? 'Add a supplier to your business directory.'
      : 'Manage contact, business and account information.'
})

const submitLabel = computed(() => {
  if (saving.value) return 'Saving...'

  if (isEditing.value) return 'Save changes'

  if (props.defaultRole === 'CUSTOMER') return 'Create customer'

  if (props.defaultRole === 'SUPPLIER') return 'Create supplier'

  return 'Create party'
})

function nullable(value: string) {
  const cleaned = value.trim()
  return cleaned || null
}

function populateForm() {
  const party = props.party

  errorMessage.value = ''

  if (party) {
    Object.assign(form, {
      party_code: party.party_code ?? '',
      name: party.name ?? '',
      roles: [...(party.roles ?? [])],
      contact_person: party.contact_person ?? '',
      phone: party.phone ?? '',
      alternate_phone: party.alternate_phone ?? '',
      email: party.email ?? '',
      gstin: party.gstin ?? '',
      pan: party.pan ?? '',
      drug_license_number: party.drug_license_number ?? '',
      billing_address: party.billing_address ?? '',
      shipping_address: party.shipping_address ?? '',
      city: party.city ?? '',
      district: party.district ?? '',
      state: party.state ?? 'Rajasthan',
      pincode: party.pincode ?? '',
      country: party.country ?? 'India',
      opening_balance: Number(party.opening_balance ?? 0),
      opening_balance_type:
        party.opening_balance_type ?? 'PAYABLE',
      credit_limit: Number(party.credit_limit ?? 0),
      credit_days: Number(party.credit_days ?? 0),
      notes: party.notes ?? '',
      is_active: party.is_active ?? true,
    })
  } else {
    Object.assign(form, {
      party_code: '',
      name: '',
      roles: [props.defaultRole],
      contact_person: '',
      phone: '',
      alternate_phone: '',
      email: '',
      gstin: '',
      pan: '',
      drug_license_number: '',
      billing_address: '',
      shipping_address: '',
      city: '',
      district: '',
      state: 'Rajasthan',
      pincode: '',
      country: 'India',
      opening_balance: 0,
      opening_balance_type:
        props.defaultRole === 'SUPPLIER'
          ? 'PAYABLE'
          : 'RECEIVABLE',
      credit_limit: 0,
      credit_days: 0,
      notes: '',
      is_active: true,
    })
  }
}

watch(
  () => [props.open, props.party, props.defaultRole] as const,
  ([open]) => {
    if (open) populateForm()
  },
  { immediate: true },
)

function toggleRole(role: PartyRole, checked: boolean) {
  if (checked) {
    if (!form.roles.includes(role)) {
      form.roles.push(role)
    }
  } else {
    form.roles = form.roles.filter(item => item !== role)
  }

  if (form.roles.length === 1) {
    form.opening_balance_type =
      form.roles[0] === 'SUPPLIER'
        ? 'PAYABLE'
        : 'RECEIVABLE'
  }
}

function getErrorMessage(error: unknown) {
  if (error && typeof error === 'object' && 'data' in error) {
    const data = (
      error as {
        data?: {
          message?: string
          error?: string
        }
      }
    ).data

    if (data?.message) return data.message
    if (data?.error) return data.error
  }

  if (error instanceof Error) return error.message

  return 'Unable to save party. Please try again.'
}

async function submitForm() {
  errorMessage.value = ''

  if (!form.name.trim()) {
    errorMessage.value = 'Party name is required.'
    return
  }

  if (!form.roles.length) {
    errorMessage.value = 'Select at least one party type.'
    return
  }

  if (
    form.email.trim() &&
    !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.email.trim())
  ) {
    errorMessage.value = 'Enter a valid email address.'
    return
  }

  if (
    form.pincode.trim() &&
    !/^\d{6}$/.test(form.pincode.trim())
  ) {
    errorMessage.value = 'Enter a valid 6-digit Indian pincode.'
    return
  }

  if (
    form.gstin.trim() &&
    !/^[0-9A-Z]{15}$/.test(form.gstin.trim().toUpperCase())
  ) {
    errorMessage.value =
      'GSTIN must contain 15 alphanumeric characters.'
    return
  }

  if (
    form.pan.trim() &&
    !/^[A-Z]{5}[0-9]{4}[A-Z]$/.test(form.pan.trim().toUpperCase())
  ) {
    errorMessage.value = 'Enter a valid PAN.'
    return
  }

  if (
    !Number.isFinite(Number(form.opening_balance)) ||
    Number(form.opening_balance) < 0
  ) {
    errorMessage.value = 'Opening balance cannot be negative.'
    return
  }

  if (
    !Number.isFinite(Number(form.credit_limit)) ||
    Number(form.credit_limit) < 0
  ) {
    errorMessage.value = 'Credit limit cannot be negative.'
    return
  }

  if (
    !Number.isInteger(Number(form.credit_days)) ||
    Number(form.credit_days) < 0
  ) {
    errorMessage.value =
      'Credit days must be a non-negative integer.'
    return
  }

  const payload: PartyInput = {
    party_code: form.party_code.trim(),
    name: form.name.trim(),
    roles: [...form.roles],

    contact_person: nullable(form.contact_person),
    phone: nullable(form.phone),
    alternate_phone: nullable(form.alternate_phone),
    email: nullable(form.email),

    gstin: nullable(form.gstin.toUpperCase()),
    pan: nullable(form.pan.toUpperCase()),
    drug_license_number: nullable(form.drug_license_number),

    billing_address: nullable(form.billing_address),
    shipping_address: nullable(form.shipping_address),
    city: nullable(form.city),
    district: nullable(form.district),
    state: form.state.trim() || 'Rajasthan',
    pincode: form.pincode.trim(),
    country: form.country.trim() || 'India',

    opening_balance: Number(form.opening_balance),
    opening_balance_type: form.opening_balance_type,
    credit_limit: Number(form.credit_limit),
    credit_days: Number(form.credit_days),

    notes: nullable(form.notes),
    is_active: form.is_active,
  }

  saving.value = true

  try {
    const savedParty = props.party
      ? await updateParty(props.party.id, payload)
      : await createParty(payload)

    emit('saved', savedParty)
    emit('close')
  } catch (error) {
    errorMessage.value = getErrorMessage(error)
  } finally {
    saving.value = false
  }
}

function closeModal() {
  if (saving.value) return
  emit('close')
}

function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.open) {
    closeModal()
  }
}

watch(
  () => props.open,
  (open) => {
    if (!import.meta.client) return

    if (open) {
      window.addEventListener('keydown', handleEscape)
    } else {
      window.removeEventListener('keydown', handleEscape)
    }
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  if (import.meta.client) {
    window.removeEventListener('keydown', handleEscape)
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="party-modal">
      <div
        v-if="open"
        class="party-overlay"
        @mousedown.self="closeModal"
      >
        <section
          class="party-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="party-form-title"
        >
          <!-- Header -->
          <header class="party-header">
            <div class="party-heading">
              <div class="party-heading-icon">
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  aria-hidden="true"
                >
                  <rect x="3" y="5" width="18" height="16" rx="3" />
                  <circle cx="9" cy="11" r="2.5" />
                  <path d="M5 18c.8-2.2 2.2-3.2 4-3.2s3.2 1 4 3.2M15 10h3M15 14h3" />
                </svg>
              </div>

              <div class="party-header-copy">
                <span class="party-eyebrow">PARTY MANAGEMENT</span>
                <h2 id="party-form-title">{{ title }}</h2>
                <p>{{ description }}</p>
              </div>
            </div>

            <button
              type="button"
              class="party-close"
              aria-label="Close form"
              :disabled="saving"
              @click="closeModal"
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                aria-hidden="true"
              >
                <path d="m6 6 12 12M18 6 6 18" />
              </svg>
            </button>
          </header>

          <!-- Scrollable form -->
          <form
            id="party-form"
            class="party-form"
            @submit.prevent="submitForm"
          >
            <div class="party-form-content">
              <!-- Error message -->
              <div
                v-if="errorMessage"
                class="party-error"
                role="alert"
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  aria-hidden="true"
                >
                  <circle cx="12" cy="12" r="9" />
                  <path d="M12 8v5M12 16.5h.01" />
                </svg>

                <span>{{ errorMessage }}</span>
              </div>

              <!-- Basic information -->
              <section class="party-section">
                <div class="party-section-heading">
                  <div class="party-section-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.8"
                      aria-hidden="true"
                    >
                      <path d="M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" />
                      <path d="M4 21v-2a8 8 0 0 1 16 0v2" />
                    </svg>
                  </div>

                  <div>
                    <h3>Basic information</h3>
                    <p>Identity and party classification</p>
                  </div>
                </div>

                <div class="party-grid">
                  <label class="party-field">
                    <span class="party-label">
                      Party name <span class="required">*</span>
                    </span>

                    <input
                      v-model="form.name"
                      class="party-input"
                      required
                      maxlength="200"
                      autocomplete="organization"
                      placeholder="Business or person's name"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Party code</span>

                    <input
                      v-model="form.party_code"
                      class="party-input"
                      maxlength="50"
                      :readonly="isEditing"
                      placeholder="Enter party code"
                    >

                    <span class="party-hint">
                      {{
                        isEditing
                          ? 'Code cannot be changed.'
                          : 'Leave blank if generated automatically.'
                      }}
                    </span>
                  </label>

                  <div class="party-field party-field-full">
                    <span class="party-label">
                      Party type <span class="required">*</span>
                    </span>

                    <div class="party-role-options">
                      <label
                        class="party-role-option"
                        :class="{
                          'party-role-option-active':
                            form.roles.includes('SUPPLIER'),
                        }"
                      >
                        <input
                          type="checkbox"
                          :checked="form.roles.includes('SUPPLIER')"
                          @change="
                            toggleRole(
                              'SUPPLIER',
                              ($event.target as HTMLInputElement).checked,
                            )
                          "
                        >

                        <span class="party-role-check">
                          <svg
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            aria-hidden="true"
                          >
                            <path d="m5 12 4 4L19 6" />
                          </svg>
                        </span>

                        <span>Supplier</span>
                      </label>

                      <label
                        class="party-role-option"
                        :class="{
                          'party-role-option-active':
                            form.roles.includes('CUSTOMER'),
                        }"
                      >
                        <input
                          type="checkbox"
                          :checked="form.roles.includes('CUSTOMER')"
                          @change="
                            toggleRole(
                              'CUSTOMER',
                              ($event.target as HTMLInputElement).checked,
                            )
                          "
                        >

                        <span class="party-role-check">
                          <svg
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            aria-hidden="true"
                          >
                            <path d="m5 12 4 4L19 6" />
                          </svg>
                        </span>

                        <span>Customer</span>
                      </label>
                    </div>

                    <span class="party-hint">
                      A party can be both a supplier and a customer.
                    </span>
                  </div>
                </div>
              </section>

              <!-- Contact information -->
              <section class="party-section">
                <div class="party-section-heading">
                  <div class="party-section-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.8"
                      aria-hidden="true"
                    >
                      <path d="M21 16.5v3a2 2 0 0 1-2.2 2A19.8 19.8 0 0 1 10.2 18a19.4 19.4 0 0 1-6-6A19.8 19.8 0 0 1 1.9 3.2 2 2 0 0 1 3.9 1h3a2 2 0 0 1 2 1.7l.5 3a2 2 0 0 1-.6 1.9L6.9 9.5a16 16 0 0 0 6 6l1.9-1.9a2 2 0 0 1 1.9-.6l3 .5a2 2 0 0 1 1.3 2Z" />
                    </svg>
                  </div>

                  <div>
                    <h3>Contact information</h3>
                    <p>Phone numbers and communication details</p>
                  </div>
                </div>

                <div class="party-grid">
                  <label class="party-field">
                    <span class="party-label">Contact person</span>

                    <input
                      v-model="form.contact_person"
                      class="party-input"
                      maxlength="150"
                      autocomplete="name"
                      placeholder="Contact person's name"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Phone</span>

                    <input
                      v-model="form.phone"
                      class="party-input"
                      maxlength="20"
                      type="tel"
                      autocomplete="tel"
                      placeholder="Primary phone number"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Alternate phone</span>

                    <input
                      v-model="form.alternate_phone"
                      class="party-input"
                      maxlength="20"
                      type="tel"
                      placeholder="Alternate phone number"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Email address</span>

                    <input
                      v-model="form.email"
                      class="party-input"
                      maxlength="254"
                      type="email"
                      autocomplete="email"
                      placeholder="name@company.com"
                    >
                  </label>
                </div>
              </section>

              <!-- Tax and licence -->
              <section class="party-section">
                <div class="party-section-heading">
                  <div class="party-section-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.8"
                      aria-hidden="true"
                    >
                      <path d="M7 3h7l5 5v13H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2Z" />
                      <path d="M14 3v6h5M9 14h6M9 18h6" />
                    </svg>
                  </div>

                  <div>
                    <h3>Tax and licence details</h3>
                    <p>Registration and compliance information</p>
                  </div>
                </div>

                <div class="party-grid">
                  <label class="party-field">
                    <span class="party-label">GSTIN</span>

                    <input
                      v-model="form.gstin"
                      class="party-input party-uppercase"
                      maxlength="15"
                      autocomplete="off"
                      placeholder="15-character GSTIN"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">PAN</span>

                    <input
                      v-model="form.pan"
                      class="party-input party-uppercase"
                      maxlength="10"
                      autocomplete="off"
                      placeholder="ABCDE1234F"
                    >
                  </label>

                  <label class="party-field party-field-full">
                    <span class="party-label">Drug licence number</span>

                    <input
                      v-model="form.drug_license_number"
                      class="party-input"
                      maxlength="100"
                      placeholder="Licence number, if applicable"
                    >
                  </label>
                </div>
              </section>

              <!-- Address -->
              <section class="party-section">
                <div class="party-section-heading">
                  <div class="party-section-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.8"
                      aria-hidden="true"
                    >
                      <path d="M20 10c0 5-8 12-8 12S4 15 4 10a8 8 0 1 1 16 0Z" />
                      <circle cx="12" cy="10" r="2.5" />
                    </svg>
                  </div>

                  <div>
                    <h3>Address</h3>
                    <p>Billing, shipping and location information</p>
                  </div>
                </div>

                <div class="party-grid">
                  <label class="party-field">
                    <span class="party-label">Billing address</span>

                    <textarea
                      v-model="form.billing_address"
                      class="party-input party-textarea"
                      rows="3"
                      maxlength="1000"
                      placeholder="Street, building and locality"
                    />
                  </label>

                  <label class="party-field">
                    <span class="party-label">Shipping address</span>

                    <textarea
                      v-model="form.shipping_address"
                      class="party-input party-textarea"
                      rows="3"
                      maxlength="1000"
                      placeholder="Leave blank if same as billing address"
                    />
                  </label>

                  <label class="party-field">
                    <span class="party-label">City</span>

                    <input
                      v-model="form.city"
                      class="party-input"
                      maxlength="100"
                      placeholder="City"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">District</span>

                    <input
                      v-model="form.district"
                      class="party-input"
                      maxlength="100"
                      placeholder="District"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">State</span>

                    <input
                      v-model="form.state"
                      class="party-input"
                      required
                      maxlength="100"
                      placeholder="State"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Pincode</span>

                    <input
                      v-model="form.pincode"
                      class="party-input"
                      inputmode="numeric"
                      maxlength="6"
                      pattern="[0-9]{6}"
                      placeholder="6-digit pincode"
                    >
                  </label>

                  <label class="party-field party-field-full">
                    <span class="party-label">Country</span>

                    <input
                      v-model="form.country"
                      class="party-input"
                      required
                      maxlength="100"
                      placeholder="Country"
                    >
                  </label>
                </div>
              </section>

              <!-- Commercial terms -->
              <section class="party-section">
                <div class="party-section-heading">
                  <div class="party-section-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.8"
                      aria-hidden="true"
                    >
                      <rect x="3" y="5" width="18" height="14" rx="2" />
                      <path d="M3 10h18M7 15h3" />
                    </svg>
                  </div>

                  <div>
                    <h3>Commercial terms</h3>
                    <p>Opening balance and credit settings</p>
                  </div>
                </div>

                <div class="party-balance-note">
                  Confirm the opening balance before entering accounting
                  transactions.
                </div>

                <div class="party-grid">
                  <label class="party-field">
                    <span class="party-label">Opening balance (₹)</span>

                    <input
                      v-model.number="form.opening_balance"
                      class="party-input"
                      type="number"
                      min="0"
                      step="0.01"
                      inputmode="decimal"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Balance type</span>

                    <select
                      v-model="form.opening_balance_type"
                      class="party-input party-select"
                    >
                      <option value="PAYABLE">Payable to supplier</option>
                      <option value="RECEIVABLE">Receivable from customer</option>
                      <option value="ADVANCE">Advance</option>
                    </select>
                  </label>

                  <label class="party-field">
                    <span class="party-label">Credit limit (₹)</span>

                    <input
                      v-model.number="form.credit_limit"
                      class="party-input"
                      type="number"
                      min="0"
                      step="0.01"
                      inputmode="decimal"
                    >
                  </label>

                  <label class="party-field">
                    <span class="party-label">Credit days</span>

                    <input
                      v-model.number="form.credit_days"
                      class="party-input"
                      type="number"
                      min="0"
                      step="1"
                      inputmode="numeric"
                    >
                  </label>
                </div>
              </section>

              <!-- Notes and status -->
              <section class="party-section">
                <div class="party-section-heading">
                  <div class="party-section-icon">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.8"
                      aria-hidden="true"
                    >
                      <path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9Z" />
                      <path d="M14 3v6h6M8 13h8M8 17h8" />
                    </svg>
                  </div>

                  <div>
                    <h3>Additional information</h3>
                    <p>Internal notes and account status</p>
                  </div>
                </div>

                <label class="party-field">
                  <span class="party-label">Notes</span>

                  <textarea
                    v-model="form.notes"
                    class="party-input party-textarea"
                    rows="3"
                    maxlength="2000"
                    placeholder="Additional business notes..."
                  />
                </label>

                <label
                  v-if="isEditing"
                  class="party-active-option"
                >
                  <input
                    v-model="form.is_active"
                    type="checkbox"
                  >

                  <span class="party-active-check">
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      aria-hidden="true"
                    >
                      <path d="m5 12 4 4L19 6" />
                    </svg>
                  </span>

                  <span class="party-active-copy">
                    <strong>Active account</strong>
                    <small>
                      Allow this party to be used in new transactions.
                    </small>
                  </span>
                </label>
              </section>
            </div>
          </form>

          <!-- Fixed footer -->
          <footer class="party-footer">
            <div class="party-footer-note">
              <span class="required">*</span>
              Required fields
            </div>

            <div class="party-footer-actions">
              <button
                type="button"
                class="party-button party-button-secondary"
                :disabled="saving"
                @click="closeModal"
              >
                Cancel
              </button>

              <button
                type="submit"
                form="party-form"
                class="party-button party-button-primary"
                :disabled="saving"
              >
                <svg
                  v-if="saving"
                  class="party-spinner"
                  viewBox="0 0 24 24"
                  fill="none"
                  aria-hidden="true"
                >
                  <circle
                    cx="12"
                    cy="12"
                    r="9"
                    stroke="currentColor"
                    stroke-width="3"
                    opacity=".25"
                  />
                  <path
                    d="M21 12a9 9 0 0 0-9-9"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                  />
                </svg>

                <svg
                  v-else
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  aria-hidden="true"
                >
                  <path d="m5 12 4 4L19 6" />
                </svg>

                {{ submitLabel }}
              </button>
            </div>
          </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.party-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  overflow-y: auto;
  background: rgb(15 23 42 / 46%);
  backdrop-filter: blur(3px);
}

.party-dialog {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 760px;
  max-height: min(900px, calc(100dvh - 40px));
  overflow: hidden;
  border: 1px solid #e5e7eb;
  border-radius: 15px;
  background: #fff;
  color: #1f2937;
  box-shadow:
    0 24px 70px rgb(0 0 0 / 17%),
    0 4px 12px rgb(0 0 0 / 6%);
  font-family: inherit;
}

/* Header */

.party-header {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 22px 24px;
  border-bottom: 1px solid #edf0ee;
  background: #fff;
}

.party-heading {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 13px;
}

.party-heading-icon {
  display: flex;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 1px solid #ccebdc;
  border-radius: 10px;
  background: #effaf4;
  color: #078454;
}

.party-heading-icon svg {
  width: 23px;
  height: 23px;
}

.party-header-copy {
  min-width: 0;
}

.party-eyebrow {
  display: block;
  margin-bottom: 4px;
  color: #078454;
  font-size: 10px;
  font-weight: 750;
  letter-spacing: 0.09em;
}

.party-header h2 {
  margin: 0;
  color: #17251e;
  font-size: 22px;
  font-weight: 750;
  line-height: 1.25;
  letter-spacing: -0.045em;
}

.party-header p {
  margin: 5px 0 0;
  color: #7b8580;
  font-size: 12px;
  line-height: 1.5;
}

.party-close {
  display: flex;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 1px solid #dfe5e1;
  border-radius: 9px;
  background: #fff;
  color: #6b756f;
  cursor: pointer;
  transition:
    background 140ms ease,
    border-color 140ms ease,
    color 140ms ease;
}

.party-close svg {
  width: 19px;
  height: 19px;
}

.party-close:hover:not(:disabled) {
  border-color: #cbd5ce;
  background: #f4f7f5;
  color: #17251e;
}

.party-close:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

/* Scrollable content */

.party-form {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  overflow: hidden;
}

.party-form-content {
  display: flex;
  flex-direction: column;
  gap: 25px;
  padding: 23px 24px 28px;
  overflow-y: auto;
  overscroll-behavior: contain;
  scrollbar-color: #cbd5ce transparent;
  scrollbar-width: thin;
}

.party-form-content::-webkit-scrollbar {
  width: 7px;
}

.party-form-content::-webkit-scrollbar-track {
  background: transparent;
}

.party-form-content::-webkit-scrollbar-thumb {
  border-radius: 20px;
  background: #d5ddd8;
}

.party-form-content::-webkit-scrollbar-thumb:hover {
  background: #b4c0b8;
}

/* Error */

.party-error {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid #f2c8c8;
  border-radius: 9px;
  background: #fff6f6;
  color: #b42318;
  font-size: 12px;
  line-height: 1.6;
}

.party-error svg {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

/* Sections */

.party-section {
  display: flex;
  flex-direction: column;
  gap: 17px;
}

.party-section-heading {
  display: flex;
  align-items: center;
  gap: 11px;
  padding-bottom: 13px;
  border-bottom: 1px solid #e9eeeb;
}

.party-section-icon {
  display: flex;
  width: 35px;
  height: 35px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 1px solid #dceee4;
  border-radius: 9px;
  background: #f1faf5;
  color: #078454;
}

.party-section-icon svg {
  width: 18px;
  height: 18px;
}

.party-section-heading h3 {
  margin: 0;
  color: #26352c;
  font-size: 13px;
  font-weight: 750;
  line-height: 1.4;
}

.party-section-heading p {
  margin: 3px 0 0;
  color: #859088;
  font-size: 11px;
  line-height: 1.4;
}

.party-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: start;
  gap: 17px 16px;
}

.party-field {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 7px;
}

.party-field-full {
  grid-column: 1 / -1;
}

.party-label {
  color: #344239;
  font-size: 11px;
  font-weight: 650;
  line-height: 1.4;
}

.required {
  color: #dc4545;
}

/* Inputs */

.party-input {
  display: block;
  width: 100%;
  min-width: 0;
  min-height: 39px;
  padding: 9px 11px;
  border: 1px solid #dce3de;
  border-radius: 8px;
  outline: none;
  background: #fff;
  color: #24342a;
  font-family: inherit;
  font-size: 12px;
  line-height: 1.45;
  transition:
    border-color 140ms ease,
    box-shadow 140ms ease,
    background 140ms ease;
}

.party-input::placeholder {
  color: #a0aaa3;
  opacity: 1;
}

.party-input:hover:not(:disabled):not(:read-only) {
  border-color: #c3d1c8;
}

.party-input:focus {
  border-color: #159765;
  box-shadow: 0 0 0 3px rgb(21 151 101 / 10%);
}

.party-input:disabled {
  cursor: not-allowed;
  background: #f5f7f5;
  color: #9ca59f;
}

.party-input:read-only {
  background: #f5f7f5;
  color: #7a857d;
}

.party-textarea {
  min-height: 84px;
  resize: vertical;
}

.party-select {
  padding-right: 32px;
  cursor: pointer;
}

.party-uppercase {
  text-transform: uppercase;
}

.party-hint {
  color: #8a958e;
  font-size: 10px;
  line-height: 1.5;
}

/* Role selection */

.party-role-options {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.party-role-option {
  position: relative;
  display: inline-flex;
  min-width: 120px;
  min-height: 39px;
  align-items: center;
  gap: 9px;
  padding: 8px 12px;
  border: 1px solid #dce3de;
  border-radius: 8px;
  background: #fff;
  color: #46534a;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color 140ms ease,
    background 140ms ease,
    color 140ms ease;
}

.party-role-option:hover {
  border-color: #b7d9c6;
  background: #f9fcfa;
}

.party-role-option input {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  opacity: 0;
}

.party-role-check {
  display: flex;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 1px solid #cbd5ce;
  border-radius: 4px;
  background: #fff;
  color: transparent;
}

.party-role-check svg {
  width: 12px;
  height: 12px;
}

.party-role-option-active {
  border-color: #159765;
  background: #f0faf4;
  color: #08794d;
}

.party-role-option-active .party-role-check {
  border-color: #138b5c;
  background: #138b5c;
  color: #fff;
}

.party-role-option input:focus-visible + .party-role-check {
  outline: 3px solid rgb(21 151 101 / 20%);
  outline-offset: 2px;
}

/* Commercial terms */

.party-balance-note {
  margin-top: -6px;
  padding: 10px 12px;
  border: 1px solid #e5eee8;
  border-radius: 7px;
  background: #f7faf8;
  color: #718077;
  font-size: 11px;
  line-height: 1.5;
}

/* Active status */

.party-active-option {
  display: flex;
  align-items: center;
  gap: 11px;
  margin-top: 2px;
  padding: 13px;
  border: 1px solid #e1e8e3;
  border-radius: 9px;
  background: #fbfdfb;
  cursor: pointer;
}

.party-active-option > input {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  opacity: 0;
}

.party-active-check {
  display: flex;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 1px solid #cbd5ce;
  border-radius: 5px;
  background: #fff;
  color: transparent;
}

.party-active-check svg {
  width: 13px;
  height: 13px;
}

.party-active-option > input:checked + .party-active-check {
  border-color: #138b5c;
  background: #138b5c;
  color: #fff;
}

.party-active-option > input:focus-visible + .party-active-check {
  outline: 3px solid rgb(21 151 101 / 20%);
  outline-offset: 2px;
}

.party-active-copy {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.party-active-copy strong {
  color: #344239;
  font-size: 12px;
  font-weight: 650;
}

.party-active-copy small {
  color: #859088;
  font-size: 10px;
  line-height: 1.5;
}

/* Footer */

.party-footer {
  display: flex;
  min-height: 65px;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 12px 24px;
  border-top: 1px solid #e8ede9;
  background: #fff;
}

.party-footer-note {
  color: #8a958e;
  font-size: 10px;
  line-height: 1.5;
}

.party-footer-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 9px;
}

.party-button {
  display: inline-flex;
  min-height: 37px;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 0 15px;
  border: 1px solid transparent;
  border-radius: 8px;
  font-family: inherit;
  font-size: 11px;
  font-weight: 700;
  white-space: nowrap;
  cursor: pointer;
  transition:
    background 140ms ease,
    border-color 140ms ease,
    box-shadow 140ms ease,
    transform 140ms ease;
}

.party-button svg {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
}

.party-button:active:not(:disabled) {
  transform: translateY(1px);
}

.party-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.party-button-secondary {
  border-color: #dce3de;
  background: #fff;
  color: #455249;
}

.party-button-secondary:hover:not(:disabled) {
  border-color: #c3cec6;
  background: #f6f8f6;
}

.party-button-primary {
  border-color: #128355;
  background: #128355;
  color: #fff;
  box-shadow: 0 2px 4px rgb(18 131 85 / 12%);
}

.party-button-primary:hover:not(:disabled) {
  border-color: #096b44;
  background: #096b44;
  box-shadow: 0 3px 8px rgb(18 131 85 / 18%);
}

.party-spinner {
  animation: party-spin 750ms linear infinite;
}

@keyframes party-spin {
  to {
    transform: rotate(360deg);
  }
}

/* Modal animation */

.party-modal-enter-active,
.party-modal-leave-active {
  transition: opacity 170ms ease;
}

.party-modal-enter-active .party-dialog,
.party-modal-leave-active .party-dialog {
  transition:
    transform 170ms ease,
    opacity 170ms ease;
}

.party-modal-enter-from,
.party-modal-leave-to {
  opacity: 0;
}

.party-modal-enter-from .party-dialog,
.party-modal-leave-to .party-dialog {
  transform: translateY(9px) scale(0.99);
  opacity: 0;
}

/* Responsive */

@media (min-width: 1200px) {
  .party-dialog {
    max-width: 780px;
  }

  .party-form-content {
    padding-right: 27px;
    padding-left: 27px;
  }
}

@media (max-width: 640px) {
  .party-overlay {
    align-items: flex-end;
    padding: 0;
  }

  .party-dialog {
    width: 100%;
    max-width: none;
    max-height: 94dvh;
    border-radius: 16px 16px 0 0;
    border-bottom: 0;
  }

  .party-header {
    padding: 16px;
    gap: 10px;
  }

  .party-heading {
    gap: 10px;
  }

  .party-heading-icon {
    width: 39px;
    height: 39px;
  }

  .party-header h2 {
    font-size: 18px;
  }

  .party-header p {
    font-size: 11px;
  }

  .party-form-content {
    gap: 23px;
    padding: 19px 16px 24px;
  }

  .party-grid {
    grid-template-columns: minmax(0, 1fr);
    gap: 15px;
  }

  .party-field-full {
    grid-column: auto;
  }

  .party-footer {
    padding: 12px 16px;
    padding-bottom: max(12px, env(safe-area-inset-bottom));
  }

  .party-footer-note {
    display: none;
  }

  .party-footer-actions {
    width: 100%;
  }

  .party-button {
    min-height: 40px;
  }

  .party-button-secondary {
    flex: 1;
  }

  .party-button-primary {
    flex: 1.5;
  }
}

@media (max-width: 360px) {
  .party-header {
    padding: 13px;
  }

  .party-form-content {
    padding-right: 13px;
    padding-left: 13px;
  }

  .party-footer {
    padding-right: 13px;
    padding-left: 13px;
  }

  .party-role-option {
    min-width: 0;
    flex: 1;
  }

  .party-button {
    padding-right: 10px;
    padding-left: 10px;
    font-size: 10px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .party-modal-enter-active,
  .party-modal-leave-active,
  .party-modal-enter-active .party-dialog,
  .party-modal-leave-active .party-dialog,
  .party-button,
  .party-input,
  .party-close {
    transition: none;
  }

  .party-spinner {
    animation-duration: 1.5s;
  }
}
</style>