import { createApp } from "vue";
import "./style.css";
import PrimeVue from "primevue/config";
import ToastService from "primevue/toastservice";
import ConfirmationService from "primevue/confirmationservice";
import Aura from "@primeuix/themes/aura";
import App from "./pages/set.vue";

createApp(App)
  .use(PrimeVue, {
    theme: {
      preset: Aura,
      options: { darkModeSelector: ".dark" },
    },
  })
  .use(ToastService)
  .use(ConfirmationService)
  .mount("#app");
