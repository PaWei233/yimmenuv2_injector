import { createApp } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from "./App.vue";

createApp(App).mount("#app");

updateTitle();

async function updateTitle() {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.setTitle("YimMenu V2 注入器");
  } catch (error) {
    console.error("设置窗口标题时出错：", error);
  }
}