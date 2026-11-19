import { defineConfig, presetWind3 } from "unocss";

export default defineConfig({
  presets: [presetWind3()],
  shortcuts: {
    "card-base": "rounded-xl border border-solid border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800",
  },
});
