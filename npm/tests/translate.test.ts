import { readFileSync } from "node:fs";

import {
	NagaError,
	parseGlsl,
	parseSpirv,
	parseWgsl,
	translate,
	validate,
	writeGlsl,
	writeHlsl,
	writeMsl,
	writeSpirv,
	writeWgsl,
} from "naga-wasm";
import { describe, expect, it } from "vitest";

const fixture = (name: string) =>
	readFileSync(new URL(`fixtures/${name}`, import.meta.url), "utf8");

const triangle = fixture("triangle.wgsl");
const color = fixture("color.frag");

describe("backends", () => {
	const module = parseWgsl(triangle);
	const info = validate(module);

	it("writes wgsl", () => {
		expect(writeWgsl(module, info)).toMatchSnapshot();
	});

	it("writes glsl", () => {
		expect(
			writeGlsl(module, info, {
				version: "300 es",
				stage: "vertex",
				entryPoint: "vs_main",
			}),
		).toMatchSnapshot();
		expect(
			writeGlsl(module, info, {
				version: "330",
				stage: "fragment",
				entryPoint: "fs_main",
			}),
		).toMatchSnapshot();
	});

	it("writes hlsl", () => {
		expect(writeHlsl(module, info)).toMatchSnapshot();
		expect(writeHlsl(module, info, { shaderModel: "6_0" })).toMatchSnapshot();
	});

	it("writes msl", () => {
		expect(writeMsl(module, info)).toMatchSnapshot();
		expect(writeMsl(module, info, { langVersion: [2, 1] })).toMatchSnapshot();
	});

	it("writes spirv that parses back", () => {
		const words = writeSpirv(module, info);
		expect(words[0]).toBe(0x07_23_02_03);

		const roundtrip = parseSpirv(words);
		expect(writeWgsl(roundtrip, validate(roundtrip))).toMatchSnapshot();
	});
});

describe("frontends", () => {
	it("accepts undefined defines", () => {
		expect(() =>
			parseGlsl(color, { stage: "fragment", defines: undefined }),
		).not.toThrow();
	});

	it("parses glsl", () => {
		const module = parseGlsl(color, { stage: "fragment" });
		expect(writeWgsl(module, validate(module))).toMatchSnapshot();
	});
});

describe("translate", () => {
	it("matches the handle api", () => {
		const module = parseWgsl(triangle);
		expect(translate({ from: "wgsl", to: "hlsl", source: triangle })).toBe(
			writeHlsl(module, validate(module)),
		);
	});

	it("returns spirv words", () => {
		const words = translate({ from: "wgsl", to: "spirv", source: triangle });
		expect(words).toBeInstanceOf(Uint32Array);
		expect(
			translate({ from: "spirv", to: "wgsl", source: words }),
		).toMatchSnapshot();
	});

	it("passes frontend options", () => {
		expect(
			translate({
				from: "glsl",
				to: "wgsl",
				source: color,
				parse: { stage: "fragment" },
			}),
		).toMatchSnapshot();
	});
});

describe("errors", () => {
	const thrown = (fn: () => unknown) => {
		try {
			fn();
		} catch (error) {
			return error;
		}
		throw new Error("expected a throw");
	};

	it("formats parse errors", () => {
		const error = thrown(() => parseWgsl("fn main( {"));
		expect(error).toBeInstanceOf(NagaError);
		expect(error).toMatchObject({
			kind: "parse",
			message: 'expected identifier, found "{"',
		});
		expect((error as NagaError).formatted).toMatchSnapshot();
	});

	it("formats validation errors", () => {
		const module = parseWgsl(
			"var<storage> data: array<f32>;\n@compute @workgroup_size(1) fn main() { data[0] = 1.0; }",
		);
		const error = thrown(() => validate(module));
		expect(error).toBeInstanceOf(NagaError);
		expect(error).toMatchObject({
			kind: "validation",
			message: "Global variable [0] 'data' is invalid",
		});
		expect((error as NagaError).formatted).toMatchSnapshot();
	});

	it("reports write errors", () => {
		const module = parseWgsl(triangle);
		const error = thrown(() =>
			writeGlsl(module, validate(module), {
				version: "300 es",
				stage: "fragment",
				entryPoint: "missing",
			}),
		);
		expect(error).toBeInstanceOf(NagaError);
		expect(error).toMatchObject({ kind: "write" });
	});
});

describe("disposal", () => {
	it("frees handles", () => {
		const module = parseWgsl(triangle);
		module.free();
		expect(() => validate(module)).toThrow("null pointer passed to rust");
	});

	it("frees handles with using", () => {
		let escaped;
		{
			using module = parseWgsl(triangle);
			escaped = module;
		}
		expect(() => validate(escaped)).toThrow("null pointer passed to rust");
	});
});
