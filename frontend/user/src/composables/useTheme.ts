import { ref, onMounted } from 'vue'

export type ThemeName = 'dark' | 'light' | 'high-contrast-dark'

export interface ThemeConfig {
  name: ThemeName
  label: string
  icon: string
}

export const THEMES: ThemeConfig[] = [
  { name: 'dark', label: '暗色主题', icon: '🌙' },
  { name: 'light', label: '亮色主题', icon: '☀️' },
  { name: 'high-contrast-dark', label: '高对比度', icon: '🔲' },
]

const THEME_STORAGE_KEY = 'rsws-user-theme'

const currentTheme = ref<ThemeName>('dark')

function loadTheme(): ThemeName {
  const saved = localStorage.getItem(THEME_STORAGE_KEY)
  if (saved === 'dark' || saved === 'light' || saved === 'high-contrast-dark') {
    return saved
  }
  return 'dark'
}

function applyTheme(theme: ThemeName) {
  document.documentElement.setAttribute('data-theme', theme)
}

// 初始化（在模块加载时执行）
const savedTheme = loadTheme()
currentTheme.value = savedTheme
applyTheme(savedTheme)

export function useTheme() {
  function setTheme(theme: ThemeName) {
    currentTheme.value = theme
    localStorage.setItem(THEME_STORAGE_KEY, theme)
    applyTheme(theme)
  }

  function resetTheme() {
    setTheme('dark')
  }

  return {
    currentTheme,
    themes: THEMES,
    setTheme,
    resetTheme,
  }
}
