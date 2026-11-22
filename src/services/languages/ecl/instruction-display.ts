import type {
  EclInstructionReference, EclInstructionSpec, NormalizedEclSemanticData
} from '../../../types'

type ParameterContract = NonNullable<EclInstructionReference['params']>[number]

interface DisplayParameter {
  name: string
  label: string
  repeated: boolean
  contract?: ParameterContract
}

const directionLabels = { in: '输入', out: '输出（可写变量）', inout: '读写变量' }

export function formatParameterDetails(contract: ParameterContract): string {
  return [
    contract.direction && directionLabels[contract.direction],
    contract.unit,
    contract.description
  ].filter(Boolean).join(' · ')
}

export function describeInstruction(
  name: string,
  map: EclInstructionSpec | undefined,
  reference: EclInstructionReference | undefined
) {
  const signature = map?.signature ?? reference?.signature ?? null
  const contracts = reference?.signature === signature ? reference.params : undefined
  const encodings = Array.from(signature?.replace(/\s|\*/g, '') ?? '')
  const count = signature === null ? map?.params.length ?? 0 : encodings.length
  const params: DisplayParameter[] = []
  for (let index = 0; index < count; index++) {
    const mapped = map?.params[index]
    const contract = contracts?.find(param => param.position === index + 1)
    const repeated = index === count - 1 && /\*\S$/.test(signature ?? '')
    const code = encodings[index]
    // Older map hints describe *D as a pointer. Here * marks a repeatable tail.
    const type = repeated ? undefined : mapped?.type ?? code
    const paramName = contract?.name ?? (repeated ? 'args' : mapped?.name) ?? `arg${index + 1}`
    params.push({
      name: paramName, repeated, contract,
      label: [type, paramName + (repeated ? '…' : '')].filter(Boolean).join(' ')
    })
  }
  const argumentsLabel = signature === null && !params.length ? '…' : params.map(p => p.label).join(', ')
  return {
    name, opcode: map?.opcode ?? reference?.opcode, section: map?.section,
    signature, fromReference: map?.signature == null && reference?.signature != null,
    label: `${name}(${argumentsLabel})`, params, contracts, reference
  }
}

type DisplayInstruction = ReturnType<typeof describeInstruction>

export function findDisplayInstruction(data: NormalizedEclSemanticData, name: string) {
  const raw = /^ins_(\d+)$/.exec(name)
  const map = raw
    ? data.instructions.find(item => item.opcode === Number(raw[1]))
    : data.instructions.find(item => item.name === name)
  const opcode = map?.opcode ?? (raw ? Number(raw[1]) : undefined)
  const reference = data.reference?.instructions.find(item => item.opcode === opcode)
  return map || reference ? describeInstruction(name, map, reference) : null
}

export function buildInstructionInsertText(instruction: DisplayInstruction): string {
  const placeholders = instruction.params.filter(p => !p.repeated).map((param, index) =>
    `\${${index + 1}:${param.name.replace(/[\\$}]/g, '\\$&')}}`
  )
  const cursor = instruction.signature === null || instruction.params.some(p => p.repeated) ? '$0' : ''
  return `${instruction.name}(${placeholders.join(', ')}${cursor});`
}

export function buildInstructionDocumentation(instruction: DisplayInstruction): { value: string } {
  const paramDocs = instruction.contracts?.map(contract => {
    const details = formatParameterDetails(contract)
    return `- 参数 ${contract.position} · \`${contract.name}\`${details ? ' · ' + details : ''}`
  })
  return { value: [
    `\`\`\`ecl\n${instruction.label}\n\`\`\``,
    [`opcode ${instruction.opcode}`, instruction.section].filter(Boolean).join(' · '),
    instruction.reference?.description,
    instruction.signature !== null
      ? `${instruction.fromReference ? '参考签名' : 'map 签名'} \`${instruction.signature || '无参数'}\`` : '',
    paramDocs?.join('\n')
  ].filter(Boolean).join('\n\n') }
}
