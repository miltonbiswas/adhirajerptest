<script setup lang="ts">
import {
  Activity,
  BarChart3,
  Boxes,
  ChevronDown,
  CircleHelp,
  Command,
  CreditCard,
  Database,
  FlaskConical,
  LayoutDashboard,
  LifeBuoy,
  Package,
  ReceiptText,
  Search,
  Settings,
  ShieldCheck,
  ShoppingCart,
  Tags,
  Truck,
  UserRound,
  Users,
  Wallet,
} from 'lucide-vue-next'

const route = useRoute()

const search = ref('')
const utilitiesExpanded = ref(false)
const settingsExpanded = ref(false)

const navigation = [
  { label: 'Dashboard', to: '/', icon: LayoutDashboard },
  { label: 'Products', to: '/products', icon: Package },
  { label: 'Inventory', to: '/inventory', icon: Boxes },
  { label: 'Purchases', to: '/purchases', icon: ShoppingCart },
  { label: 'Sales', to: '/sales', icon: CreditCard },
  { label: 'Parties', to: '/parties', icon: Users },
  { label: 'Suppliers', to: '/suppliers', icon: Truck },
  { label: 'Accounts', to: '/accounts', icon: Wallet },
  { label: 'Reports', to: '/reports', icon: BarChart3 },
]

const utilityItems = [
  { label: 'Brands', to: '/utilities/brands', icon: Tags },
  { label: 'Compositions', to: '/utilities/compositions', icon: FlaskConical },
  { label: 'Tax & GST', to: '/utilities/tax', icon: ReceiptText },
]

const settingsItems = [
  { label: 'General', to: '/settings', icon: Settings },
  { label: 'Data & Backup', to: '/settings/backup', icon: Database },
  { label: 'Security', to: '/settings/security', icon: ShieldCheck },
]

const query = computed(() => search.value.trim().toLowerCase())

const filteredNavigation = computed(() =>
  navigation.filter(item =>
    `${item.label} ${item.to}`.toLowerCase().includes(query.value),
  ),
)

const filteredUtilities = computed(() =>
  utilityItems.filter(item =>
    `${item.label} ${item.to}`.toLowerCase().includes(query.value),
  ),
)

const filteredSettings = computed(() =>
  settingsItems.filter(item =>
    `${item.label} ${item.to}`.toLowerCase().includes(query.value),
  ),
)

const showUtilities = computed(() =>
  !query.value || filteredUtilities.value.length > 0,
)

const showSettings = computed(() =>
  !query.value || filteredSettings.value.length > 0,
)

const showProfile = computed(() =>
  !query.value || 'profile business account preferences'.includes(query.value),
)

function isActive(path: string) {
  if (path === '/') return route.path === '/'
  return route.path === path || route.path.startsWith(`${path}/`)
}
</script>

<template>
  <aside
    class="flex h-full w-[248px] shrink-0 flex-col overflow-hidden border-r border-[#DCE4EF] bg-[#FBFCFE] text-[#647793]"
  >
    <!-- Brand: larger logo, no additional brand text -->
    <div
      class="flex h-[68px] shrink-0 items-center justify-start border-b border-[#DCE4EF] px-4"
    >
      <NuxtLink
        to="/"
        aria-label="AdhirajERP home"
        class="flex h-full w-full items-center"
      >
        <img
          src="/images/logo.png"
          alt="AdhirajERP"
          class="block h-[48px] w-auto max-w-[200px] object-contain object-left"
        >
      </NuxtLink>
    </div>

    <!-- Search -->
    <div class="shrink-0 px-3 py-3">
      <label
        class="flex h-[36px] items-center gap-2 rounded-lg border border-[#DCE4EF] bg-white px-2.5 transition focus-within:border-[#A9B9CC]"
      >
        <Search class="size-4 shrink-0 text-[#647793]" />

        <input
          v-model="search"
          type="search"
          placeholder="Search..."
          aria-label="Search navigation"
          autocomplete="off"
          class="min-w-0 flex-1 bg-transparent text-[12.5px] text-[#34445D] outline-none placeholder:text-[#8493A8]"
        >

        <kbd
          class="hidden items-center gap-1 rounded border border-[#D5DFEC] bg-[#F5F8FC] px-1.5 py-0.5 text-[10px] text-[#647793] sm:flex"
        >
          <Command class="size-2.5" />
          K
        </kbd>
      </label>
    </div>

    <!-- Navigation -->
    <nav class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden px-3 pb-3">
      <div class="space-y-1">
        <NuxtLink
          v-for="item in filteredNavigation"
          :key="item.to"
          :to="item.to"
          class="group flex h-[36px] items-center gap-2.5 rounded-md px-2.5 text-[12.5px] font-medium transition-colors"
          :class="
            isActive(item.to)
              ? 'bg-[#EDF2F7] text-[#008F61]'
              : 'text-[#647793] hover:bg-[#F0F4F8] hover:text-[#34445D]'
          "
        >
          <component
            :is="item.icon"
            class="size-[16px] shrink-0"
            :stroke-width="1.8"
          />

          <span class="truncate">{{ item.label }}</span>
        </NuxtLink>
      </div>

      <!-- Utilities: collapsed by default -->
      <section v-if="showUtilities" class="mt-2">
        <button
          type="button"
          class="flex h-[36px] w-full items-center gap-2.5 rounded-md px-2.5 text-[12.5px] font-medium transition-colors hover:bg-[#F0F4F8] hover:text-[#34445D]"
          :class="
            filteredUtilities.some(item => isActive(item.to))
              ? 'text-[#008F61]'
              : 'text-[#647793]'
          "
          :aria-expanded="query ? true : utilitiesExpanded"
          @click="utilitiesExpanded = !utilitiesExpanded"
        >
          <Boxes class="size-[16px] shrink-0" :stroke-width="1.8" />

          <span class="flex-1 text-left">Utilities</span>

          <ChevronDown
            class="size-3.5 shrink-0 transition-transform"
            :class="{ 'rotate-180': query || utilitiesExpanded }"
          />
        </button>

        <div
          v-if="query || utilitiesExpanded"
          class="ml-[17px] mt-1 space-y-0.5 border-l border-[#DCE4EF] pl-2"
        >
          <NuxtLink
            v-for="item in filteredUtilities"
            :key="item.to"
            :to="item.to"
            class="flex h-[32px] items-center gap-2 rounded-r-md px-2 text-[12px] transition-colors"
            :class="
              isActive(item.to)
                ? 'bg-[#EDF2F7] font-medium text-[#008F61]'
                : 'text-[#647793] hover:bg-[#F0F4F8] hover:text-[#34445D]'
            "
          >
            <component
              :is="item.icon"
              class="size-[14px] shrink-0"
              :stroke-width="1.8"
            />

            <span class="truncate">{{ item.label }}</span>
          </NuxtLink>
        </div>
      </section>

      <!-- Settings: collapsed by default -->
      <section v-if="showSettings" class="mt-1">
        <button
          type="button"
          class="flex h-[36px] w-full items-center gap-2.5 rounded-md px-2.5 text-[12.5px] font-medium transition-colors hover:bg-[#F0F4F8] hover:text-[#34445D]"
          :class="
            settingsItems.some(item => isActive(item.to))
              ? 'text-[#008F61]'
              : 'text-[#647793]'
          "
          :aria-expanded="query ? true : settingsExpanded"
          @click="settingsExpanded = !settingsExpanded"
        >
          <Settings class="size-[16px] shrink-0" :stroke-width="1.8" />

          <span class="flex-1 text-left">Settings</span>

          <ChevronDown
            class="size-3.5 shrink-0 transition-transform"
            :class="{ 'rotate-180': query || settingsExpanded }"
          />
        </button>

        <div
          v-if="query || settingsExpanded"
          class="ml-[17px] mt-1 space-y-0.5 border-l border-[#DCE4EF] pl-2"
        >
          <NuxtLink
            v-for="item in filteredSettings"
            :key="item.to"
            :to="item.to"
            class="flex h-[32px] items-center gap-2 rounded-r-md px-2 text-[12px] transition-colors"
            :class="
              isActive(item.to)
                ? 'bg-[#EDF2F7] font-medium text-[#008F61]'
                : 'text-[#647793] hover:bg-[#F0F4F8] hover:text-[#34445D]'
            "
          >
            <component
              :is="item.icon"
              class="size-[14px] shrink-0"
              :stroke-width="1.8"
            />

            <span class="truncate">{{ item.label }}</span>
          </NuxtLink>
        </div>
      </section>

      <p
        v-if="
          query
          && filteredNavigation.length === 0
          && filteredUtilities.length === 0
          && filteredSettings.length === 0
          && !showProfile
        "
        class="px-2.5 py-5 text-xs text-[#8493A8]"
      >
        No matching navigation items.
      </p>
    </nav>

    <!-- Bottom actions -->
    <div class="shrink-0 border-t border-[#DCE4EF] px-3 py-2">
      <NuxtLink
        to="/activity"
        class="flex h-[34px] items-center gap-2.5 rounded-md px-2.5 text-[12px] text-[#647793] transition-colors hover:bg-[#F0F4F8] hover:text-[#34445D]"
        :class="isActive('/activity') ? 'bg-[#EDF2F7] text-[#008F61]' : ''"
      >
        <Activity class="size-[15px]" :stroke-width="1.8" />
        Activity Log
      </NuxtLink>

      <NuxtLink
        to="/help"
        class="flex h-[34px] items-center gap-2.5 rounded-md px-2.5 text-[12px] text-[#647793] transition-colors hover:bg-[#F0F4F8] hover:text-[#34445D]"
      >
        <LifeBuoy class="size-[15px]" :stroke-width="1.8" />
        Help & Support
      </NuxtLink>

      <NuxtLink
        to="/feedback"
        class="flex h-[34px] items-center gap-2.5 rounded-md px-2.5 text-[12px] text-[#647793] transition-colors hover:bg-[#F0F4F8] hover:text-[#34445D]"
      >
        <CircleHelp class="size-[15px]" :stroke-width="1.8" />
        Feedback
      </NuxtLink>
    </div>

    <!-- Profile -->
    <div
      v-if="showProfile"
      class="shrink-0 border-t border-[#DCE4EF] px-3 py-2"
    >
      <NuxtLink
        to="/profile"
        class="group flex h-[44px] items-center gap-2.5 rounded-lg border px-2.5 transition-colors"
        :class="
          isActive('/profile')
            ? 'border-[#D8E8E0] bg-[#F0F7F3] text-[#087A55]'
            : 'border-transparent bg-[#F5F7FA] text-[#647793] hover:border-[#E1E7EF] hover:bg-white hover:text-[#34445D]'
        "
      >
        <div
          class="flex size-[28px] shrink-0 items-center justify-center rounded-full border border-[#DCE4EF] bg-white text-[#526781]"
        >
          <UserRound class="size-[15px]" :stroke-width="1.8" />
        </div>

        <div class="min-w-0 flex-1">
          <p class="truncate text-[12px] font-medium text-[#34445D]">
            Profile
          </p>

          <p class="mt-0.5 truncate text-[10px] text-[#8493A8]">
            Account & preferences
          </p>
        </div>
      </NuxtLink>
    </div>
  </aside>
</template>
