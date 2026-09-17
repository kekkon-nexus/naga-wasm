import { $, file } from "bun";

import { nagaVersion } from "../src/version.ts";

$.cwd(new URL("../../", import.meta.url).pathname);

const lockfile = await file("Cargo.lock").text();
const locked = /name = "wasm-bindgen"\nversion = "(.+)"/.exec(lockfile)?.[1];
const cliVersion = await $`wasm-bindgen --version`.text();
const cli = cliVersion.trim().split(" ")[1];
if (cli !== locked) {
	throw new Error(
		`wasm-bindgen CLI ${cli} does not match Cargo.lock ${locked}`,
	);
}

const manifest = await file("Cargo.toml").text();
const pinned = /^naga = \{ version = "=(.+?)"/m.exec(manifest)?.[1];
if (pinned !== nagaVersion) {
	throw new Error(
		`nagaVersion ${nagaVersion} does not match Cargo.toml naga ${pinned}`,
	);
}

await $`cargo build --release --target wasm32-unknown-unknown`;
await $`rm -rf npm/dist`;
await $`wasm-bindgen --target web --weak-refs --out-dir npm/dist/wasm --out-name naga target/wasm32-unknown-unknown/release/naga_wasm.wasm`;
await $`wasm-opt -Oz npm/dist/wasm/naga_bg.wasm -o npm/dist/wasm/naga_bg.wasm`;
await $`tsc -p npm`;
