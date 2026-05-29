import { createApp } from "vue";
import App from "./App.vue";

// 通用字体
import "./styles/global.css";
// 等宽字体
import 'vfonts/FiraCode.css'

const app = createApp(App);
app.mount("#app");
