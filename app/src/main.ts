import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./assets/main.css";

/**
 * 初始化 Vue 应用并挂载到根节点。
 */
const bootstrap = () => {
  const app = createApp(App);
  app.use(createPinia());
  app.mount("#app");
};

bootstrap();

