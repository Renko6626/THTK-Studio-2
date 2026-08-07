<template>
  <div class="h-full flex flex-col min-h-0 text-[12px]">
    <div v-if="!dialect" class="flex-1 flex items-center justify-center text-center text-[#9d9d9d] px-4 leading-6">
      时间线适用于 .dstd / .dmsg
      <br />
      这两种格式都是按时间编排的指令流
    </div>

    <template v-else>
      <div class="shrink-0 pb-2 text-[11px] text-[#9d9d9d] leading-5">
        {{ groups.length }} 个时间点
        <span v-if="spanCount"> · {{ spanCount }} 段插值</span>
        <span v-if="crossingCount" class="text-[#cca700]"> · {{ crossingCount }} 段跨时间点</span>
        <span v-if="analysis.hasHalt"> · 末尾为无限等待</span>
      </div>

      <div v-if="!groups.length" class="flex-1 flex items-center justify-center text-[#9d9d9d]">
        没有可显示的时间点
      </div>

      <div v-else class="flex-1 min-h-0 overflow-auto">
        <div v-for="group in groups" :key="group.time" class="mb-2">
          <!-- 时间点：带到下一点的间隔。间隔是文本给不了的信息 -->
          <div class="flex items-baseline gap-2 sticky top-0 bg-[#252526] py-0.5">
            <span class="font-mono text-[#e7e7e7]">{{ group.time }}</span>
            <span v-if="group.delta !== null" class="text-[10px] text-[#9d9d9d]">
              Δ{{ group.delta }}
            </span>
          </div>

          <div
            v-for="event in group.events"
            :key="event.line"
            class="pl-3 py-0.5 rounded-[3px] cursor-pointer hover:bg-white/8 flex items-baseline gap-1.5"
            :title="describe(event)"
            @click="reveal(event.line)"
          >
            <span class="shrink-0 w-4 text-center" :class="markClass(event.kind)">
              {{ mark(event.kind) }}
            </span>
            <span class="font-mono truncate" :class="event.kind === 'instant' ? 'text-[#9d9d9d]' : 'text-[#e7e7e7]'">
              {{ event.name }}
            </span>
            <span v-if="event.kind === 'span'" class="text-[10px] shrink-0" :class="event.crossesNextLabel ? 'text-[#cca700]' : 'text-[#4ec9b0]'">
              →{{ event.endTime }}<span v-if="event.crossesNextLabel">⇢</span>
            </span>
            <span v-else-if="event.kind === 'loop'" class="text-[10px] text-[#cca700] shrink-0">
              → {{ event.jumpTime }}
            </span>
            <span v-else-if="event.kind === 'interrupt'" class="text-[10px] text-[#c586c0] shrink-0">
              #{{ event.interruptId }}
            </span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { analyzeTimeline, dialectForPath } from '../../services/languages/std/timeline'
import type { TimelineEvent, TimelineEventKind } from '../../services/languages/std/timeline'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'

const editorStore = useEditorStore()

const activeTab = computed(() => editorStore.activeTab)
const dialect = computed(() =>
  activeTab.value?.viewType === 'text' ? dialectForPath(activeTab.value?.path) : null
)

const analysis = computed(() =>
  dialect.value
    ? analyzeTimeline(activeTab.value?.content || '', dialect.value)
    : { groups: [], hasHalt: false }
)
const groups = computed(() => analysis.value.groups)

const allEvents = computed(() => groups.value.flatMap((g) => g.events))
const spanCount = computed(() => allEvents.value.filter((e) => e.kind === 'span').length)
/** 插值非阻塞，跨时间点很常见但看文本发现不了——单独计数提醒 */
const crossingCount = computed(() => allEvents.value.filter((e) => e.crossesNextLabel).length)

/**
 * 标记要能一眼区分四类。用符号而非颜色单独承载信息——
 * 颜色在不同主题下可能失效，符号不会。
 */
function mark(kind: TimelineEventKind): string {
  switch (kind) {
    case 'span':
      return '▬'
    case 'loop':
      return '⟲'
    case 'interrupt':
      return '⚡'
    case 'halt':
      return '⏸'
    default:
      return '·'
  }
}

function markClass(kind: TimelineEventKind): string {
  switch (kind) {
    case 'span':
      return 'text-[#4ec9b0]'
    case 'loop':
      return 'text-[#cca700]'
    case 'interrupt':
      return 'text-[#c586c0]'
    case 'halt':
      return 'text-[#f48771]'
    default:
      return 'text-[#6a6a6a]'
  }
}

/** 悬停说明。这些语义来自 thtk 源码与 .stdm 的说明，用户不该被要求自己记住 */
function describe(event: TimelineEvent): string {
  switch (event.kind) {
    case 'span': {
      const base = `${event.name}：从第 ${event.time} 帧插值 ${event.duration} 帧，到第 ${event.endTime} 帧结束。插值不阻塞脚本，期间脚本继续执行。`
      return event.crossesNextLabel
        ? `${base}\n⇢ 它越过了下一个时间点——读到那里时这个动作仍在进行中。`
        : base
    }
    case 'loop':
      return `${event.name}：跳转并把当前时间设为 ${event.jumpTime}——这是倒带，不只是控制流`
    case 'interrupt':
      return `中断标签 #${event.interruptId}：可在脚本等待时被 ECL 指令触发`
    case 'halt':
      return `${event.name}：无限等待。时间插值继续更新，中断仍可触发——不是结束`
    default:
      return `${event.name}：第 ${event.line} 行`
  }
}

function reveal(line: number) {
  const path = activeTab.value?.path
  if (!path) return
  dispatchEditorRevealLocation({ path, line, column: 1 })
}
</script>
