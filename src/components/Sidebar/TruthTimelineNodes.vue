<template>
  <div class="timeline-nodes">
    <div v-for="node in nodes" :key="node.id" class="timeline-node" :data-timeline-node="node.id">
      <template v-if="node.kind === 'time'">
        <div class="timeline-time">
          <span class="timeline-dot" aria-hidden="true" />
          <button type="button" class="timeline-time-label" :title="node.source || '脚本起始时间'" @click="$emit('reveal', node.line)">{{ node.name }} 帧</button>
          <span class="timeline-time-mode">{{ node.name.trim().startsWith('+') ? '相对标记' : '源码时间' }}</span>
        </div>
        <div v-if="node.children.length" class="timeline-children">
          <TruthTimelineNodes :nodes="node.children" :collapsed="collapsed" :active-line="activeLine" @toggle="$emit('toggle', $event)" @reveal="$emit('reveal', $event)" />
        </div>
      </template>

      <template v-else-if="isBlock(node)">
        <div class="timeline-control" :class="{ 'timeline-control-current': containsCursor(node) }">
          <button type="button" class="timeline-fold" :aria-label="`${isCollapsed(node) ? '展开' : '折叠'} ${node.source || node.name}`" :aria-expanded="!isCollapsed(node)" @click="$emit('toggle', `node:${node.id}`)">
            <svg viewBox="0 0 16 16" width="14" height="14" :class="{ 'timeline-chevron-open': !isCollapsed(node) }" aria-hidden="true"><path d="m6 4 4 4-4 4" /></svg>
          </button>
          <span class="timeline-control-symbol" aria-hidden="true">{{ node.kind === 'loop' ? '↻' : '◇' }}</span>
          <button type="button" class="timeline-control-title" :title="node.source" @click="$emit('reveal', node.line)">{{ node.source || node.name }}</button>
          <span class="timeline-line">L{{ node.line }}</span>
        </div>
        <div v-if="!isCollapsed(node)" class="timeline-children timeline-block-body">
          <TruthTimelineNodes :nodes="node.children" :collapsed="collapsed" :active-line="activeLine" @toggle="$emit('toggle', $event)" @reveal="$emit('reveal', $event)" />
          <button v-if="node.kind === 'loop'" type="button" class="timeline-loop-back" :title="`定位循环控制，第 ${node.targetLine ?? node.line} 行`" @click="$emit('reveal', node.targetLine ?? node.line)">↶ {{ node.name === 'loop' ? '下一轮' : '判断下一轮' }}</button>
          <div v-if="!node.children.length" class="timeline-empty-block">空代码块</div>
        </div>
      </template>

      <button v-else-if="node.kind === 'interrupt'" type="button" class="timeline-interrupt" :title="node.source" :aria-current="containsCursor(node) ? 'location' : undefined" @click="$emit('reveal', node.line)">
        <span class="timeline-interrupt-tag">入口</span>
        <span class="timeline-interrupt-name">{{ node.name }}</span>
        <span class="timeline-line">L{{ node.line }}</span>
      </button>

      <div v-else-if="node.kind === 'jump'" class="timeline-jump">
        <button type="button" class="timeline-event timeline-jump-source" :title="node.source" :aria-current="containsCursor(node) ? 'location' : undefined" @click="$emit('reveal', node.line)">
          <span class="timeline-jump-symbol" aria-hidden="true">↳</span>
          <span class="timeline-event-text">{{ node.source || node.name }}</span>
        </button>
        <button v-if="node.targetLine !== null" type="button" class="timeline-jump-target" :title="`跳到第 ${node.targetLine} 行`" @click="$emit('reveal', node.targetLine)">L{{ node.targetLine }} ↗</button>
      </div>

      <button v-else-if="node.kind === 'label'" type="button" class="timeline-label" :title="node.source" :aria-current="containsCursor(node) ? 'location' : undefined" @click="$emit('reveal', node.line)">
        <span class="timeline-label-name">{{ node.name }}:</span><span class="timeline-line">L{{ node.line }}</span>
      </button>

      <button v-else type="button" class="timeline-event" :title="node.source" :aria-current="containsCursor(node) ? 'location' : undefined" @click="$emit('reveal', node.line)">
        <span class="timeline-event-content"><span class="timeline-event-name">{{ node.name }}</span><span class="timeline-detail timeline-source">{{ node.source }}</span></span>
        <span class="timeline-line">L{{ node.line }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { TruthTimelineNode } from '../../types'

const props = defineProps<{
  nodes: TruthTimelineNode[]
  collapsed: Set<string>
  activeLine: number | null
}>()
defineEmits<{ toggle: [key: string]; reveal: [line: number] }>()

function isBlock(node: TruthTimelineNode) { return ['loop', 'branch', 'block'].includes(node.kind) }
function isCollapsed(node: TruthTimelineNode) { return props.collapsed.has(`node:${node.id}`) }
function containsCursor(node: TruthTimelineNode) { return props.activeLine !== null && props.activeLine >= node.line && props.activeLine <= node.endLine }
</script>

<style scoped>
.timeline-nodes { min-width: 0; }
.timeline-node + .timeline-node { margin-top: 2px; }
button { appearance: none; border: 0; background: transparent; color: inherit; font: inherit; cursor: pointer; padding: 0; text-align: left; }
button:focus-visible { outline: 1px solid #0078d4; outline-offset: -1px; }
.timeline-time { display: flex; min-width: 0; align-items: center; gap: 7px; min-height: 22px; padding-top: 2px; }
.timeline-dot { width: 6px; height: 6px; flex: none; border: 1px solid #75beff; background: #252526; border-radius: 50%; }
.timeline-time-label { flex: none; color: #9cdcfe; font-size: 12px; font-variant-numeric: tabular-nums; white-space: nowrap; }
.timeline-time-label:hover { text-decoration: underline; }
.timeline-time-mode { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 10px; color: #858585; }
.timeline-children { margin-left: 3px; padding: 1px 0 3px 10px; border-left: 1px solid #454545; min-width: 0; }
.timeline-block-body { padding-top: 3px; }
.timeline-control { display: flex; min-width: 0; align-items: center; gap: 4px; min-height: 24px; color: #dcdcaa; }
.timeline-control-current { color: #fff1b8; }
.timeline-fold { display: flex; align-items: center; justify-content: center; width: 18px; height: 24px; flex: none; color: #9d9d9d; }
.timeline-fold svg { fill: none; stroke: currentColor; stroke-width: 1.5; }
.timeline-chevron-open { transform: rotate(90deg); }
.timeline-control-symbol { font-size: 17px; flex: none; }
.timeline-control-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding: 3px 0; }
.timeline-control-title:hover { text-decoration: underline; }
.timeline-line { font-size: 10px; color: #858585; white-space: nowrap; flex: none; margin-left: auto; font-variant-numeric: tabular-nums; }
.timeline-event, .timeline-label, .timeline-interrupt { display: flex; align-items: center; gap: 6px; width: 100%; min-width: 0; box-sizing: border-box; border-radius: 3px; padding: 3px 5px; }
.timeline-event:hover, .timeline-label:hover, .timeline-interrupt:hover { background: #ffffff09; }
button[aria-current="location"] { background: #04395e; color: #fff; }
.timeline-event-content { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 1px; }
.timeline-event-name, .timeline-event-text, .timeline-label-name, .timeline-interrupt-name, .timeline-detail { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.timeline-event-name { color: #dcdcaa; }
.timeline-detail { font-size: 11px; color: #9d9d9d; line-height: 14px; }
.timeline-source { font-family: Consolas, 'Fira Code', monospace; }
.timeline-interrupt { margin-top: 6px; margin-bottom: 2px; color: #ccc; border-top: 1px solid #3c3c3c; border-radius: 0; padding: 5px 3px 3px; }
.timeline-interrupt-tag { flex: none; border: 1px solid #454545; border-radius: 2px; padding: 0 3px; color: #9d9d9d; font-size: 10px; line-height: 14px; white-space: nowrap; }
.timeline-interrupt-name { flex: 1; font-family: Consolas, 'Fira Code', monospace; font-size: 11px; }
.timeline-label { color: #d7ba7d; padding-top: 5px; padding-bottom: 2px; }
.timeline-jump { display: flex; min-width: 0; align-items: center; gap: 4px; }
.timeline-jump-source { flex: 1; }
.timeline-jump-symbol { color: #c586c0; flex: none; }
.timeline-jump-target { flex: none; color: #75beff; font-size: 10px; white-space: nowrap; padding: 3px 2px; }
.timeline-jump-target:hover { text-decoration: underline; }
.timeline-loop-back { display: block; color: #a995b6; font-size: 10px; padding: 3px 5px 1px; }
.timeline-loop-back:hover { color: #c586c0; }
.timeline-empty-block { padding: 4px 6px; color: #858585; font-size: 11px; }
</style>
