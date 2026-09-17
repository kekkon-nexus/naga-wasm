import { fileURLToPath } from "node:url";

import { build, type Rolldown } from "vite";
import { expect, it } from "vitest";

it("bundles the browser entry with its wasm", async () => {
	const result = (await build({
		root: fileURLToPath(new URL("vite", import.meta.url)),
		logLevel: "silent",
		build: { write: false },
	})) as Rolldown.RolldownOutput;

	const files = result.output.map((file) => file.fileName);
	expect(files.some((file) => file.endsWith(".wasm"))).toBe(true);

	const code = result.output
		.filter((file) => file.type === "chunk")
		.map((file) => file.code)
		.join("\n");
	expect(code).not.toContain("node:fs");
});
