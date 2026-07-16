<template>
  <div v-if="enabled" class="particles">
    <div
      v-for="i in 10"
      :key="i"
      class="particles__dot"
      :style="{
        left: ((i * 17 + 31) % 100) + '%',
        animationDuration: (6 + ((i * 13) % 10)) + 's',
        animationDelay: ((i * 7) % 6) + 's',
        opacity: 0.10 + ((i * 7) % 6) * 0.01,
      }"
    ></div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ enabled: boolean }>()
</script>

<style scoped>
.particles { position: fixed; inset: 0; pointer-events: none; z-index: 0; }
.particles__dot {
  position: absolute; top: 50%;
  width: 3px; height: 3px;
  border-radius: 50%;
  background: rgba(255,255,255,0.12);
  animation: drift linear infinite;
}
@keyframes drift {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-20px); }
}
</style>
