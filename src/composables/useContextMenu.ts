import { ref } from 'vue'

export interface MenuItem {
  label: string
  action: string
  danger?: boolean
  separator?: boolean
}

export function useContextMenu() {
  const menuVisible = ref(false)
  const menuX = ref(0)
  const menuY = ref(0)
  const menuItems = ref<MenuItem[]>([])

  function openMenu(x: number, y: number, items: MenuItem[]) {
    menuX.value = x
    menuY.value = y
    menuItems.value = items
    menuVisible.value = true
  }

  function closeMenu() {
    menuVisible.value = false
  }

  return { menuVisible, menuX, menuY, menuItems, openMenu, closeMenu }
}
