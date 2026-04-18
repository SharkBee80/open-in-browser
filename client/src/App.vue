<template>
  <main class="container">
    <div class="title">
      <h1>应用设置</h1>
      <button @click="updatePort">应用并重启</button>
    </div>
    <div class="settings-card">
      <div class="setting-item">
        <label for="port-input">HTTP 服务端口:</label>
        <div class="input-group">
          <input id="port-input" v-model.number="port" type="number" placeholder="例如: 3000" />

        </div>
      </div>
      <div class="setting-item">
        <label for="key-input">鉴权 Key:</label>
        <div class="input-group">
          <input id="key-input" v-model="key" type="text" placeholder="default" />
        </div>
      </div>

      <p class="status-msg" :class="{ error: status.includes('失败') }">
        {{ status }}
      </p>

      <div class="info-section">
        <div class="info-header" @click="toggleInfo">
          <span>使用说明</span>
          <span class="info-arrow" :class="{ collapsed: isInfoCollapsed }">▼</span>
        </div>
        <transition name="info-collapse">
          <Info v-if="!isInfoCollapsed" :port></Info>
        </transition>
      </div>
    </div>
  </main>
</template>
<script setup lang="ts">
  import { ref, onMounted } from "vue";
  import { invoke } from "@tauri-apps/api/core";
  import Info from "./info.vue";

  const port = ref(52798);
  const key = ref("open-in-browser");
  const status = ref("");
  const isInfoCollapsed = ref(true); // 默认折叠

  function toggleInfo() {
    isInfoCollapsed.value = !isInfoCollapsed.value;
  }

  async function loadConfig() {
    try {
      const config = await invoke<{ port: number; key: string }>("get_config");
      port.value = config.port;
      key.value = config.key;
    } catch (e) {
      status.value = "加载配置失败: " + e;
    }
  }

  async function updatePort() {
    status.value = "正在更新配置...";
    try {
      await invoke("update_config", { port: port.value, key: key.value });
      status.value = `已更新：端口=${port.value}，key已同步`;
    } catch (e) {
      status.value = "更新失败: " + e;
    }
  }

  onMounted(() => {
    loadConfig();
  });
</script>

<style scoped>
  :global(html ::-webkit-scrollbar) {
    display: none;
  }

  .title {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .container {
    padding: 2rem;
    max-width: 600px;
    margin: 0 auto;
  }

  .settings-card {
    background: white;
    padding: 2rem;
    border-radius: 12px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    text-align: left;
  }

  .setting-item {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: bold;
  }

  .input-group {
    display: flex;
    gap: 10px;
  }

  input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }

  button {
    padding: 8px 16px;
    background-color: #24c8db;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    transition: background 0.2s;
  }

  button:hover {
    background-color: #1ea7b9;
  }

  .status-msg {
    font-size: 0.9rem;
    color: #666;
    margin-top: 1rem;
  }

  .status-msg.error {
    color: #e74c3c;
  }

  .info-section {
    margin-top: 1.5rem;
    border-top: 1px solid #eee;
  }

  .info-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0;
    cursor: pointer;
    font-weight: bold;
    color: #333;
    user-select: none;
    transition: color 0.2s;
  }

  .info-header:hover {
    color: #24c8db;
  }

  .info-arrow {
    transition: transform 0.3s ease;
    font-size: 0.8rem;
  }

  .info-arrow.collapsed {
    transform: rotate(-90deg);
  }

  .info-collapse-enter-active,
  .info-collapse-leave-active {
    transition: all 0.3s ease;
    overflow: hidden;
    max-height: 500px;
    opacity: 1;
  }

  .info-collapse-enter-from,
  .info-collapse-leave-to {
    max-height: 0;
    opacity: 0;
    margin-top: 0;
    padding-top: 0;
  }

  .info {
    margin-top: 0.5rem;
    padding-top: 0.5rem;
  }

  .info h3 {
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  code {
    display: block;
    background: #f4f4f4;
    padding: 10px;
    border-radius: 4px;
    font-family: monospace;
    word-break: break-all;
  }
</style>
