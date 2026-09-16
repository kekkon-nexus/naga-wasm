import oxlint from "@kekkon-nexus/config/oxlint";
import vitest from "@kekkon-nexus/config/oxlint/vitest";
import { defineConfig } from "oxlint";

export default defineConfig({
	extends: [oxlint, vitest],
	env: {
		"builtin": true,
		"shared-node-browser": true,
		"browser": true,
		"node": true,
	},
	options: {
		typeAware: true,
		typeCheck: true,
	},
});
