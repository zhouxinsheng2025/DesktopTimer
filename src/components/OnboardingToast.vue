<template>
  <div v-if="visible" class="onboard">
    <div class="onboard__card">
      <div class="onboard__text">开机时自动启动？</div>
      <div class="onboard__actions">
        <button class="onboard__btn onboard__btn--yes" @click="$emit('enable')">开启</button>
        <button class="onboard__btn onboard__btn--no" @click="$emit('dismiss')">暂不</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean }>()
defineEmits<{ enable: []; dismiss: [] }>()
</script>

<style scoped>
.onboard { padding: 12px 8px 0; -webkit-app-region: no-drag; }
.onboard__card {
  background: rgba(20,22,30,0.94);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 10px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}
.onboard__text { color: rgba(255,255,255,0.85); font-size: 14px; }
.onboard__actions { display: flex; gap: 10px; }
.onboard__btn {
  padding: 6px 20px; border: none; border-radius: 6px;
  font-size: 13px; cursor: pointer;
}
.onboard__btn--yes { background: rgba(0,180,255,0.2); color: #00b4ff; }
.onboard__btn--no { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.5); }
</style>
