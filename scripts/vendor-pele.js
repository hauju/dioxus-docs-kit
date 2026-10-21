// Ship the lockfile-pinned ESM modules intact so relative lazy imports work.
// Consumers build with Cargo only; Bun is needed solely when updating Pele.
import { mkdir, readdir, copyFile, rm } from 'node:fs/promises';
const source = 'node_modules/pele';
const target = 'crates/dioxus-mdx/assets/pele';
const { version } = await Bun.file(source + '/package.json').json();
await rm(target, { recursive: true, force: true });
await mkdir(target + '/chunks', { recursive: true });
const files = ['lazy.js', ...(await readdir(source + '/dist/esm/chunks')).filter(name => name.endsWith('.js')).sort().map(name => 'chunks/' + name)];
for (const file of files) await copyFile(source + '/dist/esm/' + file, target + '/' + file);
await copyFile(source + '/LICENSE', target + '/LICENSE');
const license = (await Bun.file(source + '/LICENSE').text()).trim();
await Bun.write(target + '/lazy.js', `/*! Pele ${version}\n${license}\n*/\n` + await Bun.file(target + '/lazy.js').text());
await Bun.write(target + '/README.txt', `Pele ${version} — https://pele.run/\nVendored by bun run vendor:pele. See LICENSE.\n`);
