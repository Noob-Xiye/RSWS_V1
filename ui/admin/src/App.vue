<template>
  <router-view v-if="!authStore.isLoggedIn" />
  <ModernLayout v-else>
    <router-view />
  </ModernLayout>
</template>

<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
import ModernLayout from '@/components/ModernLayout.vue'

const authStore = useAuthStore()
</script>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f0f2f5; }
</style>