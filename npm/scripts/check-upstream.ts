import { appendFile } from "node:fs/promises";

import { file, semver, write } from "bun";

import { nagaVersion } from "../src/version.ts";

process.chdir(new URL("../../", import.meta.url).pathname);

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
	const [latestMajor, latestMinor] = latest.split(".");
	const [currentMajor, currentMinor] = nagaVersion.split(".");
	const kind =
		latestMajor === currentMajor
			? latestMinor === currentMinor
				? "patch"
				: "minor"
			: "major";

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

	const packageJson = await file("npm/package.json").text();
	await write(
		"npm/package.json",
		packageJson.replace(
			/"version": "(\d+)\.(\d+)\.(\d+)"/,
			(_, major: string, minor: string, patch: string) => {
				const version =
					kind === "major"
						? `${latestMajor}.0.0`
						: kind === "minor"
							? `${major}.${Number(minor) + 1}.0`
							: `${major}.${minor}.${Number(patch) + 1}`;
				return `"version": "${version}"`;
			},
		),
	);

	console.log(`naga ${nagaVersion} -> ${latest} (${kind})`);
	if (process.env["GITHUB_OUTPUT"]) {
		await appendFile(
			process.env["GITHUB_OUTPUT"],
			`version=${latest}\nkind=${kind}\n`,
		);
	}
} else {
	console.log(`naga ${nagaVersion} is up to date`);
}
