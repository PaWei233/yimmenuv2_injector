<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Gta5 {
  platform: string;
  game_path: string;
  fsl_path: string | null;
  is_running: boolean; // 新增字段
}

const gta5Info = ref<Gta5 | null>(null);
const errorMessage = ref("");
const injectMsg = ref("");

async function fetchGta5Info() {
  try {
    gta5Info.value = await invoke<Gta5>("get_gta5_info");
  } catch (error) {
    gta5Info.value = null;
    errorMessage.value = `获取 GTA5 信息失败: ${error}`;
  }
}

async function injectGTAV() {
  try {
    await invoke("inject_gta5");
    injectMsg.value = "注入成功！";
  } catch (error) {
    injectMsg.value = `注入失败：${error}`;
  }
}

async function install_fsl() {
  try {
    await invoke("install_fsl");
    injectMsg.value = "安装 FSL 成功！";
  } catch (error) {
    injectMsg.value = `安装 FSL 失败：${error}`;
  }
}

async function run_gta5() {
  try {
    await invoke("run_gta5");
    injectMsg.value = "启动 GTA5 成功！";
  } catch (error) {
    injectMsg.value = `启动 GTA5 失败：${error}`;
  }
}

let intervalId: number | null = null;

onMounted(() => {
  // 每秒调用一次后端命令
  intervalId = setInterval(fetchGta5Info, 1000);
});

onUnmounted(() => {
  // 清除定时器
  if (intervalId !== null) {
    clearInterval(intervalId);
  }
});
</script>

<template>
  <main class="container">
    <h1>欢迎使用YimMenu V2</h1>
    <div>
      <table class="info-table">
        <tbody>
          <tr>
            <td>GTA5 状态:</td>
            <td>
              <span :class="gta5Info ? 'text-green' : 'text-red'">
                {{ gta5Info ? "已安装" : "未安装" }}
              </span>
            </td>
          </tr>
          <tr>
            <td>FSL 状态:</td>
            <td>
              <span :class="gta5Info && gta5Info.fsl_path ? 'text-green' : 'text-red'">
                {{ (gta5Info && gta5Info.fsl_path) ? "已安装" : "未安装" }}
              </span>
            </td>
          </tr>
          <tr>
            <td>GTA5 运行状态:</td>
            <td>
              <span :class="gta5Info && gta5Info.is_running ? 'text-green' : 'text-red'">
                {{ (gta5Info && gta5Info.is_running) ? "正在运行" : "未运行" }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <p v-if="errorMessage" style="color: red;">{{ errorMessage }}</p>

    <div class = "menu">
      <p>{{ injectMsg }}</p>
      <div class="row menu-button">
        <button v-if="gta5Info && !gta5Info.fsl_path && !gta5Info.is_running" @click="install_fsl">安装 FSL</button>
        <button v-if="gta5Info && gta5Info.fsl_path && !gta5Info.is_running" @click="run_gta5">启动 GTA</button>
        <button v-if="gta5Info && gta5Info.fsl_path && gta5Info.is_running" @click="injectGTAV">注入 Yim</button>
      </div>
    </div>
    <a href="https://gra1nbuds.top" class="bottom-right-text" target="_blank">
      by. 洛小满
    </a>
  </main>
</template>

<style scoped>
.info-table {
  margin: 0 auto;
  border-collapse: collapse;
  width: 50%;
  text-align: left;
}

.info-table td {
  padding: 8px 12px;
  border-bottom: 1px solid #ddd;
}

.info-table td:first-child {
  font-weight: bold;
  color: #333;
}

.info-table td:last-child {
  text-align: right;
}

.text-green {
  color: green;
}

.text-red {
  color: red;
}

.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

.bottom-right-text {
  position: fixed;
  bottom: 10px;
  right: 10px;
  font-size: 14px;
  color: #0f0f0f;
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.menu {
  position: fixed;
  bottom: 20px;
  left: 0;
  width: 100%;
}

.row {
  display: flex;
  justify-content: center;
}

.menu-button {
  gap: 10px;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>