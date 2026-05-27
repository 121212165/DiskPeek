/**
 * DiskPeek i18n 字符串表
 * 当前仅支持中文，结构化设计便于未来扩展英文
 */

const strings = {
  // Toolbar
  'toolbar.brand': 'DiskPeek',
  'scan.start': '开始扫描',
  'scan.scanning': '扫描中...',
  'scan.rescan': '重新扫描',
  'view.list': '列表',
  'view.treemap': '矩形树图',
  'view.sunburst': '旭日图',
  'scan.cancel': '取消扫描',

  // StatusBar
  'status.scanning': '正在扫描… 已发现 {count} 个文件 · {size}',
  'status.done': '扫描完成 · {time} · {count} 个文件 · {size}',
  'status.waiting': '等待扫描',

  // ListView
  'list.empty': '暂无扫描数据',
  'list.scanning': '正在扫描文件系统...',
  'list.col.name': '名称',
  'list.col.size': '大小',
  'list.col.time': '修改时间',
  'list.col.percent': '占比',

  // Chart views
  'chart.empty': '暂无扫描数据',
  'chart.all': '全部',
  'chart.others': '其他',

  // Time
  'time.just': '刚刚',
  'time.minutes': '{n} 分钟前',
  'time.hours': '{n} 小时前',
  'time.days': '{n} 天前',
  'time.months': '{n} 个月前',
  'time.years': '{n} 年前',

  // File types
  'filetype.video': '视频',
  'filetype.image': '图片',
  'filetype.document': '文档',
  'filetype.archive': '压缩包',
  'filetype.audio': '音频',
  'filetype.other': '其他',

  // Errors
  'error.scanFailed': '扫描失败',
  'error.cancelFailed': '取消失败',
  'error.openFailed': '无法打开文件',
  'error.cacheFailed': '缓存检查失败',
} as const;

export type StringKey = keyof typeof strings;

/**
 * 获取本地化字符串，支持简单模板变量替换
 * t('status.scanning', { count: '1234', size: '1.5 GB' })
 */
export function t(key: StringKey, vars?: Record<string, string | number>): string {
  let str = strings[key];
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      str = str.replace(`{${k}}`, String(v));
    }
  }
  return str;
}

/**
 * 获取本地化字符串（简写别名）
 */
export const _ = t;
