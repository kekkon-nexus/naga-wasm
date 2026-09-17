import { $, file, TOML } from "bun";

import { nagaVersion } from "../src/version.ts";

process.chdir(new URL("../../", import.meta.url).pathname);

const lockfile = TOML.parse(await file("Cargo.lock").text()) as {
	package: { name: string; version: string }[];
};
const locked = (name: string) =>
	lockfile.package.find((crate) => crate.name === name)?.version;

const cliVersion = await $`wasm-bindgen --version`.text();
const cli = cliVersion.trim().split(" ")[1];
if (cli !== locked("wasm-bindgen")) {
	throw new Error(
		`wasm-bindgen CLI ${cli} does not match Cargo.lock ${locked("wasm-bindgen")}`,
	);
}

if (nagaVersion !== locked("naga")) {
	throw new Error(
		`nagaVersion ${nagaVersion} does not match Cargo.lock naga ${locked("naga")}`,
	);
}

await $`cargo build --release --target wasm32-unknown-unknown`;
await $`wasm-bindgen --target web --weak-refs --out-dir npm/dist/wasm target/wasm32-unknown-unknown/release/naga.wasm`;
await $`wasm-opt -Oz npm/dist/wasm/naga_bg.wasm -o npm/dist/wasm/naga_bg.wasm`;
await $`tsc -p npm`;
await $`cp LICENSE-MIT LICENSE-APACHE npm`;
