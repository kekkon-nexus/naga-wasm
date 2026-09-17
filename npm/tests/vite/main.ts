import { init, translate } from "naga-wasm";

await init();

document.body.textContent = translate({
	from: "wgsl",
	to: "glsl",
	source: "@fragment fn main() -> @location(0) vec4<f32> { return vec4(1.0); }",
	options: { version: "300 es", stage: "fragment", entryPoint: "main" },
});
