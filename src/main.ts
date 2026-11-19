import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import App from "./App.vue";
import "virtual:uno.css";
import "./style.css";

createApp(App).use(createPinia()).use(naive).mount("#app");
