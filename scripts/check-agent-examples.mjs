import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const usage = 'node scripts/check-agent-examples.mjs --checker PATH [--truth-dir DIR]'

function main() {
  const options = new Map()
  for (let index = 2; index < process.argv.length; index += 2) {
    const name = process.argv[index]
    if (name === '--help') { console.log(usage); return }
    if (!['--checker', '--truth-dir'].includes(name) || !process.argv[index + 1]) {
      throw new Error(usage)
    }
    options.set(name, process.argv[index + 1])
  }
  const checker = options.get('--checker') || process.env.THTK_CHECK
  if (!checker) throw new Error(usage)
  const examplesDir = resolve(root, 'docs/agent/examples')
  const examples = JSON.parse(readFileSync(resolve(examplesDir, 'manifest.json'), 'utf8'))
  for (const example of examples) {
    const args = ['check', resolve(examplesDir, example.path), '--kind', example.kind,
      '--game', String(example.gameVersion), '--json']
    if (example.encoding) args.push('--encoding', example.encoding)
    if (options.has('--truth-dir')) args.push('--truth-dir', resolve(options.get('--truth-dir')))
    const run = spawnSync(resolve(checker), args, { encoding: 'utf8' })
    if (run.error) throw run.error
    if (run.status !== 0) throw new Error(`${example.path}: ${run.stdout}\n${run.stderr}`)
    const { result } = JSON.parse(run.stdout)
    if (!result.success || result.outputPath !== null) {
      throw new Error(`${example.path}: unexpected check result ${run.stdout}`)
    }
    console.log(`PASS ${example.path} (th${example.gameVersion}, ${example.kind})`)
  }
}

try { main() } catch (error) {
  console.error(error.message)
  process.exitCode = 1
}
