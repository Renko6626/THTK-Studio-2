// Build the GUI-independent checker using Studio's Rust toolchain.
import { spawnSync } from 'node:child_process'
import { mkdirSync, copyFileSync, chmodSync, rmSync } from 'node:fs'
import { resolve, join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
function run(command, args, capture = false) {
  const result = spawnSync(command, args, { cwd: root, stdio: capture ? 'pipe' : 'inherit', encoding: 'utf8' })
  if (result.status !== 0) throw new Error(`${command} failed: ${result.error ?? result.stderr ?? result.status}`)
  return result.stdout?.trim() ?? ''
}
const host = run('rustc', ['-vV'], true).match(/^host: (.+)$/m)?.[1]
const target = process.argv[2] || process.env.TAURI_ENV_TARGET_TRIPLE || host
if (!target) throw new Error('Cannot determine checker build target')
const args = target.includes('windows-msvc') && process.platform !== 'win32' ? ['xwin'] : []
args.push('build', '--manifest-path', 'crates/script-check/Cargo.toml', '--release', '--locked', '--target', target, '--bin', 'thtk-check')
run('cargo', args)
const staging = join(root, 'src-tauri/resources/script-check')
const name = `thtk-check${target.includes('windows') ? '.exe' : ''}`
mkdirSync(staging, { recursive: true })
rmSync(join(staging, name.endsWith('.exe') ? 'thtk-check' : 'thtk-check.exe'), { force: true })
const targetDir = process.env.CARGO_TARGET_DIR ? resolve(root, process.env.CARGO_TARGET_DIR) : join(root, 'crates/script-check/target')
copyFileSync(join(targetDir, target, 'release', name), join(staging, name))
if (process.platform !== 'win32') chmodSync(join(staging, name), 0o755)
console.log(`Prepared ${name} for ${target}`)
