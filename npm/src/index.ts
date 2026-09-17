export {
	default as init,
	parseWgsl,
	validate,
	writeGlsl,
} from "./wasm/naga.js";
export type { Module, ModuleInfo } from "./wasm/naga.js";
export { nagaVersion } from "./version.js";
