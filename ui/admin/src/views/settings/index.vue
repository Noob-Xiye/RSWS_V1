<template>
  <div class="page-placeholder">
    <el-empty description="系统设置开发中...">
      <el-button type="primary" @click="$router.push('/dashboard')">返回首页</el-button>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.page-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}
</style>
