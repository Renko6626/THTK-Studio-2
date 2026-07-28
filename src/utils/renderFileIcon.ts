import { h } from 'vue'
import type { VNode } from 'vue'
import type { FileNode } from '../types'

import folderIcon from 'material-icon-theme/icons/folder.svg?url'
import folderOpenIcon from 'material-icon-theme/icons/folder-open.svg?url'
import documentIcon from 'material-icon-theme/icons/document.svg?url'
import threeDIcon from 'material-icon-theme/icons/3d.svg?url'
import cppIcon from 'material-icon-theme/icons/cpp.svg?url'
import imageIcon from 'material-icon-theme/icons/image.svg?url'
import videoIcon from 'material-icon-theme/icons/video.svg?url'
import emailIcon from 'material-icon-theme/icons/email.svg?url'
import jsonIcon from 'material-icon-theme/icons/json.svg?url'
import markdownIcon from 'material-icon-theme/icons/markdown.svg?url'
import xmlIcon from 'material-icon-theme/icons/xml.svg?url'
import vueIcon from 'material-icon-theme/icons/vue.svg?url'
import javascriptIcon from 'material-icon-theme/icons/javascript.svg?url'
import typescriptIcon from 'material-icon-theme/icons/typescript.svg?url'
import powershellIcon from 'material-icon-theme/icons/powershell.svg?url'
import consoleIcon from 'material-icon-theme/icons/console.svg?url'
import zipIcon from 'material-icon-theme/icons/zip.svg?url'

/**
 * 图标只依赖这三个字段。用结构化子集而不是完整 FileNode，
 * 是因为文件树里传进来的可能是 naive-ui 的 TreeOption 包装体。
 */
type IconNode = Partial<Pick<FileNode, 'name' | 'is_dir' | 'extension'>>

function getExt(node: IconNode | null | undefined): string {
  if (node?.extension) return String(node.extension).toLowerCase()
  const name = node?.name || ''
  const index = name.lastIndexOf('.')
  return index > 0 ? name.slice(index + 1).toLowerCase() : ''
}

function resolveIconUrl(node: IconNode | null | undefined, expanded = false): string {
  if (node?.is_dir) {
    return expanded ? folderOpenIcon : folderIcon
  }

  switch (getExt(node)) {
    case 'decl':
    case 'ecl':
      return cppIcon
    case 'anm':
    case 'danm':
      return videoIcon
    case 'msg':
    case 'dmsg':
      return emailIcon
    case 'std':
    case 'dstd':
      return threeDIcon
    case 'dat':
      return zipIcon
    case 'json':
      return jsonIcon
    case 'xml':
    case 'yaml':
    case 'yml':
    case 'ini':
      return xmlIcon
    case 'md':
    case 'txt':
    case 'log':
      return markdownIcon
    case 'js':
      return javascriptIcon
    case 'ts':
      return typescriptIcon
    case 'vue':
      return vueIcon
    case 'png':
    case 'jpg':
    case 'jpeg':
    case 'bmp':
    case 'gif':
      return imageIcon
    case 'ps1':
      return powershellIcon
    case 'cmd':
    case 'bat':
    case 'sh':
      return consoleIcon
    default:
      return documentIcon
  }
}

function renderIcon(url: string, size = 18): VNode {
  return h('img', {
    src: url,
    alt: '',
    draggable: false,
    style: {
      width: `${size}px`,
      height: `${size}px`,
      display: 'block',
      objectFit: 'contain',
      userSelect: 'none'
    }
  })
}

export function getFileIconUrl(node: IconNode | null | undefined, expanded = false): string {
  return resolveIconUrl(node, expanded)
}

export function renderFileIcon(node: IconNode | null | undefined, expanded = false): VNode {
  return renderIcon(resolveIconUrl(node, expanded), 18)
}

export function renderCompactFileIcon(node: IconNode | null | undefined): VNode {
  return renderIcon(resolveIconUrl(node, false), 16)
}
