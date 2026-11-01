<script setup lang="ts">
useHead({
  title: 'Business Profile',
})

import {
  UserRound,
  Building2,
  MapPin,
  Phone,
  Mail,
  Save,
  RefreshCw,
  ShieldCheck,
  FileText,
  AlertCircle,
  CheckCircle2
} from 'lucide-vue-next'

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

function emptyProfile(): BusinessProfile {
  return {
    business_name: '',
    legal_name: '',
    business_type: 'Pharmacy / Medical Store',
    owner_name: '',
    phone: '',
    email: '',
    address_line1: '',
    address_line2: '',
    city: '',
    district: '',
    state: 'Rajasthan',
    pincode: '',
    country: 'India',
    gstin: '',
    drug_license_retail: '',
    drug_license_wholesale: '',
    pharmacy_registration: '',
  }
}

const form = reactive<BusinessProfile>(emptyProfile())

const loading = ref(false)
const saving = ref(false)
const error = ref('')
const success = ref('')
const loaded = ref(false)

function notify(message: string, isError = false) {
  error.value = isError ? message : ''
  success.value = isError ? '' : message
}

async function loadProfile() {
  loading.value = true
  error.value = ''
  success.value = ''

  try {
    const result = await $fetch<Partial<BusinessProfile>>(
      `${API}/business-profile`,
    )

    Object.assign(form, emptyProfile(), result)
    loaded.value = true
  } catch (e: any) {
    loaded.value = false

    notify(
      e?.data?.message ||
      e?.data?.error ||
      e?.message ||
      'Business profile load nahi ho saka. Backend endpoint check karo.',
      true,
    )
  } finally {
    loading.value = false
  }
}

async function saveProfile() {
  if (!form.business_name.trim()) {
    notify('Business name required hai.', true)
    return
  }

  if (form.pincode && !/^\d{6}$/.test(form.pincode)) {
    notify('PIN code 6 digits ka hona chahiye.', true)
    return
  }

  if (form.gstin && !/^[0-9A-Z]{15}$/.test(form.gstin.toUpperCase())) {
    notify('GSTIN format verify karo. GSTIN 15 characters ka hota hai.', true)
    return
  }

  saving.value = true
  error.value = ''
  success.value = ''

  try {
    const result = await $fetch<Partial<BusinessProfile>>(
      `${API}/business-profile`,
      {
        method: 'PUT',
        body: {
          ...form,
          gstin: form.gstin.trim().toUpperCase(),
        },
      },
    )

    Object.assign(form, emptyProfile(), result)
    loaded.value = true
    notify('Business profile saved successfully.')
  } catch (e: any) {
    notify(
      e?.data?.message ||
      e?.data?.error ||
      e?.message ||
      'Profile save failed. Backend endpoint aur database persistence verify karo.',
      true,
    )
  } finally {
    saving.value = false
  }
}

onMounted(loadProfile)
</script>

<template>
  <main class="min-h-full bg-[#F8F9FB] p-4 text-[#172033] sm:p-5">
    <header class="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div>
        <div class="mb-1 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.15em] text-[#647793]">
          <UserRound class="size-3.5" />
          Account / Business profile
        </div>

        <h1 class="text-[24px] font-semibold tracking-tight">Business profile</h1>

        <p class="mt-1 text-[12px] text-[#748198]">
          Business identity, contact information and licence details.
        </p>
      </div>

      <button
        class="inline-flex h-9 items-center gap-2 rounded-lg border border-[#DCE4EF] bg-white px-3 text-[12px] font-medium hover:bg-[#F7F9FC] disabled:opacity-50"
        :disabled="loading"
        @click="loadProfile"
      >
        <RefreshCw class="size-3.5" :class="{ 'animate-spin': loading }" />
        Reload
      </button>
    </header>

    <div
      v-if="error || success"
      class="mb-4 flex items-start gap-2 rounded-lg border p-3 text-[12px]"
      :class="error
        ? 'border-red-200 bg-red-50 text-red-700'
        : 'border-emerald-200 bg-emerald-50 text-emerald-700'"
    >
      <AlertCircle v-if="error" class="mt-0.5 size-4 shrink-0" />
      <CheckCircle2 v-else class="mt-0.5 size-4 shrink-0" />
      <span>{{ error || success }}</span>
    </div>

    <form class="space-y-4" @submit.prevent="saveProfile">
      <!-- Business identity -->
      <section class="rounded-xl border border-[#E1E7EF] bg-white">
        <header class="flex items-center gap-3 border-b border-[#E8ECF2] px-4 py-3.5">
          <div class="flex size-9 items-center justify-center rounded-lg border border-[#E1E7EF] bg-[#F7F9FC]">
            <Building2 class="size-4 text-[#526781]" />
          </div>

          <div>
            <h2 class="text-[13px] font-semibold">Business information</h2>
            <p class="mt-0.5 text-[11px] text-[#8490A3]">
              Legal identity and business type.
            </p>
          </div>
        </header>

        <div class="grid gap-4 p-4 sm:grid-cols-2 xl:grid-cols-3">
          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Business / trade name *
            </span>
            <input
              v-model="form.business_name"
              required
              maxlength="200"
              placeholder="Business name"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Legal name
            </span>
            <input
              v-model="form.legal_name"
              maxlength="200"
              placeholder="Registered legal name"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Business type
            </span>
            <select v-model="form.business_type" class="profile-input">
              <option>Pharmacy / Medical Store</option>
              <option>Wholesale Pharmacy</option>
              <option>Retail Pharmacy</option>
              <option>Distributor</option>
              <option>Other</option>
            </select>
          </label>

          <label class="block sm:col-span-2 xl:col-span-1">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Owner / authorised person
            </span>
            <input
              v-model="form.owner_name"
              maxlength="150"
              placeholder="Owner name"
              class="profile-input"
            >
          </label>
        </div>
      </section>

      <!-- Contact -->
      <section class="rounded-xl border border-[#E1E7EF] bg-white">
        <header class="flex items-center gap-3 border-b border-[#E8ECF2] px-4 py-3.5">
          <div class="flex size-9 items-center justify-center rounded-lg border border-[#E1E7EF] bg-[#F7F9FC]">
            <Phone class="size-4 text-[#526781]" />
          </div>

          <div>
            <h2 class="text-[13px] font-semibold">Contact details</h2>
            <p class="mt-0.5 text-[11px] text-[#8490A3]">
              Business contact information.
            </p>
          </div>
        </header>

        <div class="grid gap-4 p-4 sm:grid-cols-2">
          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Business phone
            </span>
            <input
              v-model="form.phone"
              type="tel"
              maxlength="20"
              placeholder="Contact number"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Business email
            </span>
            <input
              v-model="form.email"
              type="email"
              maxlength="254"
              placeholder="business@example.com"
              class="profile-input"
            >
          </label>
        </div>
      </section>

      <!-- Address -->
      <section class="rounded-xl border border-[#E1E7EF] bg-white">
        <header class="flex items-center gap-3 border-b border-[#E8ECF2] px-4 py-3.5">
          <div class="flex size-9 items-center justify-center rounded-lg border border-[#E1E7EF] bg-[#F7F9FC]">
            <MapPin class="size-4 text-[#526781]" />
          </div>

          <div>
            <h2 class="text-[13px] font-semibold">Business address</h2>
            <p class="mt-0.5 text-[11px] text-[#8490A3]">
              Registered or operating address.
            </p>
          </div>
        </header>

        <div class="grid gap-4 p-4 sm:grid-cols-2 xl:grid-cols-3">
          <label class="block sm:col-span-2">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Address line 1
            </span>
            <input
              v-model="form.address_line1"
              maxlength="200"
              placeholder="Shop / building / street"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Address line 2
            </span>
            <input
              v-model="form.address_line2"
              maxlength="200"
              placeholder="Area / landmark"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Village / City
            </span>
            <input
              v-model="form.city"
              maxlength="100"
              placeholder="City or village"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              District
            </span>
            <input
              v-model="form.district"
              maxlength="100"
              placeholder="District"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              State
            </span>
            <input
              v-model="form.state"
              maxlength="100"
              placeholder="State"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              PIN code
            </span>
            <input
              v-model="form.pincode"
              inputmode="numeric"
              maxlength="6"
              placeholder="6-digit PIN"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Country
            </span>
            <input
              v-model="form.country"
              maxlength="100"
              class="profile-input"
            >
          </label>
        </div>
      </section>

      <!-- Compliance -->
      <section class="rounded-xl border border-[#E1E7EF] bg-white">
        <header class="flex items-center gap-3 border-b border-[#E8ECF2] px-4 py-3.5">
          <div class="flex size-9 items-center justify-center rounded-lg border border-[#E1E7EF] bg-[#F7F9FC]">
            <ShieldCheck class="size-4 text-[#526781]" />
          </div>

          <div>
            <h2 class="text-[13px] font-semibold">Tax & pharmacy licences</h2>
            <p class="mt-0.5 text-[11px] text-[#8490A3]">
              Reference details for business records and invoices.
            </p>
          </div>
        </header>

        <div class="grid gap-4 p-4 sm:grid-cols-2">
          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              GSTIN
            </span>
            <input
              v-model="form.gstin"
              maxlength="15"
              placeholder="15-character GSTIN"
              class="profile-input uppercase"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Retail drug licence number
            </span>
            <input
              v-model="form.drug_license_retail"
              maxlength="100"
              placeholder="Retail DL number"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Wholesale drug licence number
            </span>
            <input
              v-model="form.drug_license_wholesale"
              maxlength="100"
              placeholder="Wholesale DL number"
              class="profile-input"
            >
          </label>

          <label class="block">
            <span class="mb-1.5 block text-[11px] font-medium text-[#46556C]">
              Pharmacist registration number
            </span>
            <input
              v-model="form.pharmacy_registration"
              maxlength="100"
              placeholder="Registration number"
              class="profile-input"
            >
          </label>
        </div>

        <div class="flex items-start gap-2 border-t border-[#E8ECF2] bg-[#FAFBFD] px-4 py-3">
          <FileText class="mt-0.5 size-4 shrink-0 text-[#8490A3]" />
          <p class="text-[10px] leading-5 text-[#748198]">
            These fields store reference information only. Saving a licence
            number does not validate or grant a licence.
          </p>
        </div>
      </section>

      <!-- Save -->
      <footer class="sticky bottom-0 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-[#E1E7EF] bg-white/95 px-4 py-3 backdrop-blur">
        <p class="text-[11px] text-[#8490A3]">
          {{ loaded ? 'Profile loaded from the connected API.' : 'Waiting for profile API connection.' }}
        </p>

        <button
          type="submit"
          :disabled="saving || loading"
          class="inline-flex h-9 items-center gap-2 rounded-lg bg-[#087A55] px-4 text-[12px] font-medium text-white hover:bg-[#066746] disabled:opacity-50"
        >
          <Save class="size-4" />
          {{ saving ? 'Saving...' : 'Save business profile' }}
        </button>
      </footer>
    </form>
  </main>
</template>

<style scoped>
.profile-input {
  width: 100%;
  height: 38px;
  border: 1px solid #dce4ef;
  border-radius: 8px;
  padding: 0 11px;
  font-size: 12px;
  color: #273449;
  outline: none;
  background: white;
}

.profile-input::placeholder {
  color: #9aa5b5;
}

.profile-input:focus {
  border-color: #8da99a;
  box-shadow: 0 0 0 2px rgb(8 122 85 / 7%);
}
</style>