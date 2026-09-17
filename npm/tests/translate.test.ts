import { readFileSync } from "node:fs";

import { parseWgsl, validate, writeGlsl } from "naga-wasm";
import { expect, it } from "vitest";

const triangle = readFileSync(
	new URL("fixtures/triangle.wgsl", import.meta.url),
	"utf8",
);

it("translates wgsl to glsl", () => {
	const module = parseWgsl(triangle);
	const info = validate(module);
	const glsl = writeGlsl(module, info, {
		version: "300 es",
		stage: "fragment",
		entryPoint: "fs_main",
	});
	expect(glsl).toMatchSnapshot();
});
