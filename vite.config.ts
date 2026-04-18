import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
// @ts-expect-error - Package uses CJS export style
import cesium from "vite-plugin-cesium-build";

export default defineConfig({
  plugins: [
    cesium({
      iife: false,
      customCesiumBaseUrl: "/cesium-package",
    }),
    sveltekit(),
  ],
  define: {
    CESIUM_BASE_URL: JSON.stringify("/cesium-package"),
  },
});
