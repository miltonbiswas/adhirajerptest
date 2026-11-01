<!-- app/components/layout/AppHeader.vue -->

<script setup lang="ts">
import {
  Bell,
  ChevronDown,
  Check,
  Settings,
  ShieldCheck,
  Info,
  X,
  LayoutDashboard,
  Package,
  Boxes,
  ShoppingCart,
  CreditCard,
  Users,
  Truck,
  Wallet,
  BarChart3,
  FileText,
  Plus,
  LifeBuoy,
  Search,
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()

const headerRef = ref<HTMLElement | null>(null)

const notificationsOpen = ref(false)
const profileOpen = ref(false)
const workspaceOpen = ref(false)
const aboutOpen = ref(false)
const searchOpen = ref(false)
const searchQuery = ref('')

const navigation = [
  { label: 'Overview', to: '/', icon: LayoutDashboard },
  { label: 'Products', to: '/products', icon: Package },
  { label: 'Inventory', to: '/inventory', icon: Boxes },
  { label: 'Purchases', to: '/purchases', icon: ShoppingCart },
  { label: 'Sales', to: '/sales', icon: CreditCard },
  { label: 'Parties', to: '/parties', icon: Users },
  { label: 'Suppliers', to: '/suppliers', icon: Truck },
  { label: 'Accounts', to: '/accounts', icon: Wallet },
  { label: 'Reports', to: '/reports', icon: BarChart3 },
]

const createActions = [
  {
    label: 'New Sale',
    description: 'Create a sales invoice',
    to: '/sales',
    icon: CreditCard,
  },
  {
    label: 'New Purchase',
    description: 'Record a supplier invoice',
    to: '/purchases',
    icon: ShoppingCart,
  },
  {
    label: 'Add Product',
    description: 'Add a product to the catalogue',
    to: '/products',
    icon: Package,
  },
  {
    label: 'Manage Inventory',
    description: 'View stock and batches',
    to: '/inventory',
    icon: Boxes,
  },
]

const isActive = (path: string) => {
  if (path === '/') {
    return route.path === '/'
  }

  return route.path === path || route.path.startsWith(`${path}/`)
}

function closeMenus() {
  notificationsOpen.value = false
  profileOpen.value = false
  workspaceOpen.value = false
  searchOpen.value = false
}

function toggleNotifications() {
  notificationsOpen.value = !notificationsOpen.value
  profileOpen.value = false
  workspaceOpen.value = false
  searchOpen.value = false
}

function toggleProfile() {
  profileOpen.value = !profileOpen.value
  notificationsOpen.value = false
  workspaceOpen.value = false
  searchOpen.value = false
}

function toggleWorkspace() {
  workspaceOpen.value = !workspaceOpen.value
  notificationsOpen.value = false
  profileOpen.value = false
  searchOpen.value = false
}

function toggleSearch() {
  searchOpen.value = !searchOpen.value
  notificationsOpen.value = false
  profileOpen.value = false
  workspaceOpen.value = false

  if (!searchOpen.value) {
    searchQuery.value = ''
  }
}

async function goTo(path: string) {
  closeMenus()
  aboutOpen.value = false

  if (route.path !== path) {
    await router.push(path)
  }
}

function handleSearch() {
  const query = searchQuery.value.trim().toLowerCase()

  if (!query) return

  const match = navigation.find(item =>
    item.label.toLowerCase().includes(query),
  )

  if (match) {
    goTo(match.to)
    return
  }

  const allRoutes = [
    ...navigation,
    { label: 'Business Profile', to: '/profile', icon: Settings },
    { label: 'Settings', to: '/settings', icon: Settings },
    { label: 'Brands', to: '/utilities/brands', icon: Boxes },
    {
      label: 'Compositions',
      to: '/utilities/compositions',
      icon: FileText,
    },
  ]

  const found = allRoutes.find(item =>
    item.label.toLowerCase().includes(query),
  )

  if (found) {
    goTo(found.to)
  }
}

function handleOutsideClick(event: MouseEvent) {
  if (
    headerRef.value &&
    event.target instanceof Node &&
    !headerRef.value.contains(event.target)
  ) {
    closeMenus()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closeMenus()
    aboutOpen.value = false
  }

  if (
    (event.ctrlKey || event.metaKey) &&
    event.key.toLowerCase() === 'k'
  ) {
    event.preventDefault()
    toggleSearch()
  }
}

onMounted(() => {
  document.addEventListener('click', handleOutsideClick)
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleOutsideClick)
  document.removeEventListener('keydown', handleKeydown)
})

watch(
  () => route.fullPath,
  () => {
    closeMenus()
    aboutOpen.value = false
  },
)
</script>

<template>
  <header
    ref="headerRef"
    class="relative z-[100] shrink-0 border-b border-[#E6EAF0] bg-white"
  >
    <!-- Main navigation row -->
    <div
      class="flex h-[68px] min-w-0 items-center gap-3 px-3 sm:px-5 lg:px-6"
    >
      <!-- Floating pill navigation -->
      <nav
        aria-label="Main navigation"
        class="flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto
               rounded-full bg-[#F2F4F7] p-1
               [scrollbar-width:none]
               [&::-webkit-scrollbar]:hidden"
      >
        <NuxtLink
          v-for="item in navigation"
          :key="item.to"
          :to="item.to"
          :aria-current="isActive(item.to) ? 'page' : undefined"
          class="flex h-[38px] shrink-0 items-center justify-center gap-2
                 rounded-full px-3 text-[12px] font-medium
                 transition-all duration-200 sm:px-3.5"
          :class="
            isActive(item.to)
              ? 'bg-[#111827] text-white shadow-sm'
              : 'text-[#596579] hover:bg-white hover:text-[#172033]'
          "
        >
          <component
            :is="item.icon"
            class="size-[15px] shrink-0"
            :stroke-width="1.8"
          />

          <span>{{ item.label }}</span>
        </NuxtLink>
      </nav>

      <!-- Right-side actions -->
      <div class="flex shrink-0 items-center gap-0.5 sm:gap-1">
        <!-- Search -->
        <div class="relative">
          <button
            type="button"
            aria-label="Search navigation"
            title="Search (Ctrl+K)"
            :aria-expanded="searchOpen"
            class="flex size-9 items-center justify-center rounded-full
                   text-[#596579] transition hover:bg-[#F2F4F7]
                   hover:text-[#172033]"
            @click="toggleSearch"
          >
            <Search class="size-[18px]" :stroke-width="1.8" />
          </button>

          <section
            v-if="searchOpen"
            class="absolute right-0 top-[46px] z-[120] w-[min(320px,calc(100vw-24px))]
                   rounded-xl border border-[#E0E5EC] bg-white p-3
                   shadow-[0_16px_45px_rgba(15,23,42,0.13)]"
          >
            <form
              class="flex items-center gap-2 rounded-lg border
                     border-[#DCE3EC] px-3"
              @submit.prevent="handleSearch"
            >
              <Search class="size-4 shrink-0 text-[#8490A2]" />

              <input
                v-model="searchQuery"
                autofocus
                type="search"
                placeholder="Search menu..."
                class="h-10 min-w-0 flex-1 bg-transparent text-sm
                       text-[#172033] outline-none
                       placeholder:text-[#98A2B3]"
              />

              <button
                type="submit"
                class="text-xs font-medium text-[#087A55]"
              >
                Go
              </button>
            </form>

            <p class="px-1 pt-2 text-[11px] text-[#8490A2]">
              Search for a section, such as Products or Inventory.
            </p>
          </section>
        </div>

        <!-- Quick create -->
        <div class="relative">
          <button
            type="button"
            aria-label="Create new"
            title="Create new"
            :aria-expanded="workspaceOpen"
            class="flex size-9 items-center justify-center rounded-full
                   text-[#596579] transition hover:bg-[#F2F4F7]
                   hover:text-[#172033]"
            @click="toggleWorkspace"
          >
            <Plus
              class="size-[19px]"
              :stroke-width="1.8"
            />
          </button>

          <section
            v-if="workspaceOpen"
            class="absolute right-0 top-[46px] z-[120] w-64 overflow-hidden
                   rounded-xl border border-[#E0E5EC] bg-white p-1.5
                   shadow-[0_16px_45px_rgba(15,23,42,0.13)]"
          >
            <p
              class="px-3 py-2 text-[10px] font-semibold uppercase
                     tracking-[0.12em] text-[#929BAD]"
            >
              Quick actions
            </p>

            <button
              v-for="action in createActions"
              :key="action.to"
              type="button"
              class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5
                     text-left transition hover:bg-[#F4F6F8]"
              @click="goTo(action.to)"
            >
              <span
                class="flex size-9 shrink-0 items-center justify-center
                       rounded-lg border border-[#DDE6E1] bg-[#F0F7F3]
                       text-[#087A55]"
              >
                <component
                  :is="action.icon"
                  class="size-[17px]"
                  :stroke-width="1.8"
                />
              </span>

              <span class="min-w-0">
                <span
                  class="block text-[12px] font-medium text-[#263244]"
                >
                  {{ action.label }}
                </span>

                <span
                  class="mt-0.5 block text-[10px] text-[#8490A2]"
                >
                  {{ action.description }}
                </span>
              </span>
            </button>
          </section>
        </div>

        <!-- Notifications -->
        <div class="relative">
          <button
            type="button"
            aria-label="Notifications"
            title="Notifications"
            :aria-expanded="notificationsOpen"
            class="relative flex size-9 items-center justify-center
                   rounded-full text-[#596579] transition
                   hover:bg-[#F2F4F7] hover:text-[#172033]"
            @click="toggleNotifications"
          >
            <Bell
              class="size-[18px]"
              :stroke-width="1.8"
            />

            <span
              class="absolute right-[8px] top-[7px] size-1.5
                     rounded-full bg-[#EA580C] ring-2 ring-white"
            />
          </button>

          <section
            v-if="notificationsOpen"
            class="absolute right-0 top-[46px] z-[120]
                   w-[min(320px,calc(100vw-24px))] overflow-hidden
                   rounded-xl border border-[#E0E5EC] bg-white
                   shadow-[0_16px_45px_rgba(15,23,42,0.13)]"
          >
            <div
              class="flex items-center justify-between border-b
                     border-[#E9EDF2] px-4 py-3"
            >
              <h2 class="text-[13px] font-semibold text-[#263244]">
                Notifications
              </h2>

              <button
                type="button"
                aria-label="Close notifications"
                class="flex size-7 items-center justify-center
                       rounded-md text-[#8490A2] hover:bg-[#F2F4F7]"
                @click="notificationsOpen = false"
              >
                <X class="size-4" />
              </button>
            </div>

            <div class="px-4 py-8 text-center">
              <div
                class="mx-auto flex size-10 items-center justify-center
                       rounded-full bg-[#F0F7F3] text-[#087A55]"
              >
                <Check class="size-5" />
              </div>

              <p class="mt-3 text-[13px] font-medium text-[#263244]">
                No notifications
              </p>

              <p class="mt-1 text-[11px] text-[#8490A2]">
                You're all caught up.
              </p>
            </div>
          </section>
        </div>

        <!-- Profile -->
        <div class="relative ml-1">
          <button
            type="button"
            aria-label="Open profile menu"
            title="Profile"
            :aria-expanded="profileOpen"
            class="flex items-center gap-1.5 rounded-full p-0.5
                   transition hover:bg-[#F2F4F7]"
            @click="toggleProfile"
          >
            <span
              class="flex size-[31px] items-center justify-center
                     rounded-full border border-[#DCE4EF] bg-[#E9EEF5]
                     text-[10px] font-semibold text-[#526176]"
            >
              AD
            </span>

            <ChevronDown
              class="hidden size-3 text-[#8490A2] sm:block"
            />
          </button>

          <section
            v-if="profileOpen"
            class="absolute right-0 top-[46px] z-[120] w-60 overflow-hidden
                   rounded-xl border border-[#E0E5EC] bg-white py-1
                   shadow-[0_16px_45px_rgba(15,23,42,0.13)]"
          >
            <div class="border-b border-[#E9EDF2] px-4 py-3">
              <p class="text-[13px] font-semibold text-[#263244]">
                AdhirajERP
              </p>

              <p class="mt-1 text-[11px] text-[#8490A2]">
                Business workspace
              </p>
            </div>

            <button
              type="button"
              class="flex w-full items-center gap-3 px-4 py-2.5
                     text-left text-[12px] text-[#526176]
                     transition hover:bg-[#F4F6F8]"
              @click="goTo('/profile')"
            >
              <Settings class="size-4" />
              Business Profile
            </button>

            <button
              type="button"
              class="flex w-full items-center gap-3 px-4 py-2.5
                     text-left text-[12px] text-[#526176]
                     transition hover:bg-[#F4F6F8]"
              @click="goTo('/settings')"
            >
              <Settings class="size-4" />
              Settings
            </button>

            <button
              type="button"
              class="flex w-full items-center gap-3 px-4 py-2.5
                     text-left text-[12px] text-[#526176]
                     transition hover:bg-[#F4F6F8]"
              @click="goTo('/settings/security')"
            >
              <ShieldCheck class="size-4" />
              Security
            </button>

            <div class="my-1 border-t border-[#E9EDF2]" />

            <button
              type="button"
              class="flex w-full items-center gap-3 px-4 py-2.5
                     text-left text-[12px] text-[#526176]
                     transition hover:bg-[#F4F6F8]"
              @click="profileOpen = false; aboutOpen = true"
            >
              <Info class="size-4" />
              About AdhirajERP
            </button>
          </section>
        </div>
      </div>
    </div>
  </header>

  <!-- About dialog -->
  <Teleport to="body">
    <div
      v-if="aboutOpen"
      class="fixed inset-0 z-[200] flex items-center justify-center
             bg-[#182230]/25 p-4 backdrop-blur-[2px]"
      @click.self="aboutOpen = false"
    >
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-title"
        class="w-full max-w-sm overflow-hidden rounded-xl
               border border-[#E0E5EC] bg-white shadow-2xl"
      >
        <div
          class="flex items-center justify-between border-b
                 border-[#E9EDF2] px-5 py-4"
        >
          <h2
            id="about-title"
            class="text-sm font-semibold text-[#263244]"
          >
            About AdhirajERP
          </h2>

          <button
            type="button"
            aria-label="Close"
            class="flex size-7 items-center justify-center rounded-md
                   text-[#8490A2] hover:bg-[#F2F4F7]"
            @click="aboutOpen = false"
          >
            <X class="size-4" />
          </button>
        </div>

        <div class="px-5 py-5">
          <div class="flex items-center gap-3">
            <div
              class="flex size-11 items-center justify-center
                     rounded-xl bg-[#071B17] text-sm font-bold text-white"
            >
              A
            </div>

            <div>
              <p class="text-sm font-semibold text-[#263244]">
                AdhirajERP
              </p>

              <p class="mt-1 text-xs text-[#718198]">
                Wholesale Management System
              </p>
            </div>
          </div>

          <div
            class="mt-5 flex items-center justify-between border-t
                   border-[#E9EDF2] pt-4 text-xs"
          >
            <span class="text-[#718198]">Version</span>
            <span class="font-medium text-[#263244]">0.1.0</span>
          </div>
        </div>

        <div
          class="flex justify-end border-t border-[#E9EDF2]
                 bg-[#FBFCFE] px-5 py-3"
        >
          <button
            type="button"
            class="rounded-lg bg-[#111827] px-4 py-2
                   text-xs font-medium text-white transition
                   hover:bg-[#263244]"
            @click="aboutOpen = false"
          >
            Close
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>