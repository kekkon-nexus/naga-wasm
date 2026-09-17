import * as wasm from "./wasm/naga.js";
import type {
	GlslParseOptions,
	GlslWriteOptions,
	HlslWriteOptions,
	Module,
	ModuleInfo,
	MslWriteOptions,
	SpirvWriteOptions,
	WgslWriteOptions,
} from "./wasm/naga.js";

export { default as init } from "./wasm/naga.js";
export type {
	GlslOutput,
	GlslParseOptions,
	GlslReflection,
	GlslWriteOptions,
	HlslWriteOptions,
	InitInput,
	Module,
	ModuleInfo,
	MslWriteOptions,
	ResourceBinding,
	ShaderStage,
	SpirvWriteOptions,
	WgslWriteOptions,
} from "./wasm/naga.js";
export { nagaVersion } from "./version.js";

export type NagaErrorKind = "parse" | "validation" | "write";

export class NagaError extends Error {
	override readonly name = "NagaError";
	readonly kind: NagaErrorKind;
	readonly formatted: string;

	constructor(kind: NagaErrorKind, formatted: string) {
		super(formatted.split("\n", 1)[0]?.replace(/^error: /, ""));
		this.kind = kind;
		this.formatted = formatted;
	}
}

function wrap<Args extends unknown[], Result>(
	kind: NagaErrorKind,
	fn: (...args: Args) => Result,
) {
	return (...args: Args): Result => {
		try {
			return fn(...args);
		} catch (error) {
			// wasm-bindgen reports use-after-free with a plain Error too
			if (
				!(error instanceof Error) ||
				error instanceof WebAssembly.RuntimeError ||
				error.message === "null pointer passed to rust"
			) {
				throw error;
			}
			throw new NagaError(kind, error.message);
		}
	};
}

export const parseWgsl = wrap("parse", wasm.parseWgsl);
export const parseGlsl = wrap("parse", wasm.parseGlsl);
const parseSpirvBytes = wrap("parse", wasm.parseSpirv);
export const validate = wrap("validation", wasm.validate);
export const writeWgsl = wrap("write", wasm.writeWgsl);
export const writeGlsl = wrap("write", wasm.writeGlsl);
export const writeHlsl = wrap("write", wasm.writeHlsl);
export const writeMsl = wrap("write", wasm.writeMsl);
export const writeSpirv = wrap("write", wasm.writeSpirv);

export function parseSpirv(words: Uint8Array | Uint32Array): Module {
	return parseSpirvBytes(
		words instanceof Uint32Array
			? new Uint8Array(words.buffer, words.byteOffset, words.byteLength)
			: words,
	);
}

type TranslateFrom =
	| { from: "wgsl"; source: string }
	| { from: "glsl"; source: string; parse: GlslParseOptions }
	| { from: "spirv"; source: Uint8Array | Uint32Array };

type TranslateTo =
	| { to: "wgsl"; options?: WgslWriteOptions }
	| { to: "spirv"; options?: SpirvWriteOptions }
	| { to: "glsl"; options: GlslWriteOptions }
	| { to: "hlsl"; options?: HlslWriteOptions }
	| { to: "msl"; options?: MslWriteOptions };

export type TranslateInput = TranslateFrom & TranslateTo;

function parse(input: TranslateFrom): Module {
	switch (input.from) {
		case "wgsl": {
			return parseWgsl(input.source);
		}
		case "glsl": {
			return parseGlsl(input.source, input.parse);
		}
		case "spirv": {
			return parseSpirv(input.source);
		}
	}
}

function write(
	module: Module,
	info: ModuleInfo,
	output: TranslateTo,
): string | Uint32Array {
	switch (output.to) {
		case "wgsl": {
			return writeWgsl(module, info, output.options);
		}
		case "glsl": {
			return writeGlsl(module, info, output.options).code;
		}
		case "hlsl": {
			return writeHlsl(module, info, output.options);
		}
		case "msl": {
			return writeMsl(module, info, output.options);
		}
		case "spirv": {
			return writeSpirv(module, info, output.options);
		}
	}
}

export function translate(input: TranslateInput & { to: "spirv" }): Uint32Array;
export function translate(
	input: TranslateInput & { to: Exclude<TranslateTo["to"], "spirv"> },
): string;
export function translate(input: TranslateInput): string | Uint32Array {
	const module = parse(input);
	try {
		const info = validate(module);
		try {
			return write(module, info, input);
		} finally {
			info.free();
		}
	} finally {
		module.free();
	}
}
