import { createApp } from "vue";
import "./style.css";
import PrimeVue from "primevue/config";
import Aura from "@primeuix/themes/aura";
import App from "./pages/popup.vue";

createApp(App)
  .use(PrimeVue, {
    theme: {
      preset: Aura,
      options: { darkModeSelector: ".dark" },
    },
  })
  .mount("#app");
