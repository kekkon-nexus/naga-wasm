import { appendFile } from "node:fs/promises";

import { file, semver, write } from "bun";

import { nagaVersion } from "../npm/src/version.ts";

process.chdir(new URL("../", import.meta.url).pathname);

const response = await fetch("https://crates.io/api/v1/crates/naga", {
	headers: {
		"User-Agent": "naga-wasm (https://github.com/kekkon-nexus/naga-wasm)",
	},
});
if (!response.ok) {
	throw new Error(`crates.io responded with ${response.status}`);
}
const body = (await response.json()) as {
	crate: { max_stable_version: string };
};
const latest = body.crate.max_stable_version;

if (semver.order(latest, nagaVersion) > 0) {
	const kind =
		latest.split(".")[0] === nagaVersion.split(".")[0] ? "minor" : "major";

	const manifest = await file("Cargo.toml").text();
	await write(
		"Cargo.toml",
		manifest.replace(
			`package = "naga", version = "=${nagaVersion}"`,
			`package = "naga", version = "=${latest}"`,
		),
	);
	await write(
		"npm/src/version.ts",
		`export const nagaVersion = "${latest}";\n`,
	);

	console.info(`naga ${nagaVersion} -> ${latest} (${kind})`);
	if (process.env["GITHUB_OUTPUT"]) {
		await appendFile(
			process.env["GITHUB_OUTPUT"],
			`version=${latest}\nkind=${kind}\n`,
		);
	}
} else {
	console.info(`naga ${nagaVersion} is up to date`);
}
