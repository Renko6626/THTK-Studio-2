// Build the fixed external tools. This script is for developers/packaging; users get binaries.
import { spawnSync } from 'node:child_process'
import { mkdirSync, existsSync, readFileSync, writeFileSync, copyFileSync, chmodSync, rmSync, readdirSync } from 'node:fs'
import { resolve, join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createHash } from 'node:crypto'
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const pin = JSON.parse(readFileSync(join(root, 'deps/manifest.source.json'))).truth
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex')
function run(cmd, args, cwd = root, capture = false) {
  const p = spawnSync(cmd, args, { cwd, stdio: capture ? 'pipe' : 'inherit', encoding: 'utf8' })
  if (p.status !== 0) throw new Error(`${cmd} ${args.join(' ')} failed: ${p.error ?? p.stderr ?? p.status}`)
  return p.stdout?.trim() ?? ''
}
const compiler = '1.85.1'
const host = run('rustc', [`+${compiler}`, '-vV'], root, true).match(/^host: (.+)$/m)?.[1]
const target = process.argv[2] || process.env.TAURI_ENV_TARGET_TRIPLE || host
if (!target) throw new Error('Cannot determine truth build target')
const cache = join(root, 'deps/.cache/truth')
const staging = join(root, 'src-tauri/resources/truth')
const patch = join(root, 'deps/truth/patches/0001-msg-binary-encoding.patch')
const patchHash = hash(patch)
const stamp = join(staging, 'build-info.json')
const suffix = target.includes('windows') ? '.exe' : ''
const names = ['truth-core', 'trumsg', 'trustd'].map(n => n + suffix)
if (existsSync(stamp)) {
  const previous = JSON.parse(readFileSync(stamp))
  if (previous.packagingRevision === 2 && existsSync(join(staging, 'licenses/index.json')) && previous.baseCommit === pin.baseCommit && previous.patchSha256 === patchHash && previous.target === target && names.every(n => existsSync(join(staging, n)) && hash(join(staging, n)) === previous.sha256?.[n])) {
    console.log(`truth ${pin.baseCommit.slice(0, 7)} already prepared for ${target}`)
    process.exit(0)
  }
}
mkdirSync(dirname(cache), { recursive: true })
if (!existsSync(join(cache, '.git'))) {
  run('git', ['clone', '--no-checkout', pin.repository, cache])
  run('git', ['checkout', '--detach', pin.baseCommit], cache)
}
const current = run('git', ['rev-parse', 'HEAD'], cache, true)
if (current !== pin.baseCommit) {
  if (run('git', ['status', '--porcelain'], cache, true)) throw new Error('truth cache contains edits; move it aside before changing the pinned source')
  run('git', ['checkout', '--detach', pin.baseCommit], cache)
}
const reverse = spawnSync('git', ['apply', '--reverse', '--check', patch], { cwd: cache, stdio: 'ignore' })
if (reverse.status !== 0) run('git', ['apply', patch], cache)
const args = [`+${compiler}`]
if (target.includes('windows-msvc') && process.platform !== 'win32') args.push('xwin')
args.push('build', '--release', '--locked', '--target', target, '--bin', 'truth-core', '--bin', 'trumsg', '--bin', 'trustd')
run('cargo', args, cache)
mkdirSync(staging, { recursive: true })
// Staging contains only generated tool files; don't bundle another platform's old executables.
for (const n of ['truth-core', 'trumsg', 'trustd', 'truth-core.exe', 'trumsg.exe', 'trustd.exe']) {
  if (!names.includes(n)) rmSync(join(staging, n), { force: true })
}
for (const n of names) {
  copyFileSync(join(cache, 'target', target, 'release', n), join(staging, n))
  if (process.platform !== 'win32') chmodSync(join(staging, n), 0o755)
}
copyFileSync(join(cache, 'LICENSE'), join(staging, 'LICENSE.truth'))
// Carry the upstream license/notice texts and crate attribution with the binaries.
const metadata = JSON.parse(run('cargo', [`+${compiler}`, 'metadata', '--locked', '--format-version', '1', '--filter-platform', target], cache, true))
const licenseDir = join(staging, 'licenses')
rmSync(licenseDir, { recursive: true, force: true })
mkdirSync(licenseDir, { recursive: true })
const attributions = metadata.packages.map(pkg => {
  const directory = dirname(pkg.manifest_path)
  const destination = join(licenseDir, `${pkg.name}-${pkg.version}`)
  mkdirSync(destination, { recursive: true })
  const files = readdirSync(directory, { withFileTypes: true }).filter(entry => entry.isFile() && /^(licen[cs]e|copying|notice|unlicense|readme)([._-]|$)/i.test(entry.name)).map(entry => entry.name)
  for (const file of files) copyFileSync(join(directory, file), join(destination, file))
  copyFileSync(pkg.manifest_path, join(destination, 'Cargo.toml'))
  return { name: pkg.name, version: pkg.version, license: pkg.license, authors: pkg.authors, repository: pkg.repository, files }
})
writeFileSync(join(licenseDir, 'index.json'), JSON.stringify(attributions, null, 2) + '\n')
writeFileSync(join(staging, 'NOTICE.txt'), `truth ${pin.baseCommit} (Apache-2.0), with Studio's MSG/END binary-encoding patch.\nDependency attribution and upstream license texts are in licenses/.\nSource: ${pin.repository}\nPatch: deps/truth/patches/0001-msg-binary-encoding.patch in the Studio source distribution.\n`)
writeFileSync(stamp, JSON.stringify({ packagingRevision: 2, repository: pin.repository, baseCommit: pin.baseCommit, rustToolchain: compiler, patchSha256: patchHash, target, sha256: Object.fromEntries(names.map(n => [n, hash(join(staging, n))])) }, null, 2) + '\n')
console.log(`Prepared truth tools in ${staging}`)
