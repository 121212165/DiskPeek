import { describe, it, expect } from 'vitest';
import { formatSize, formatTime, formatRelativeTime, getFileTypeIcon, formatPercent } from '../format';

describe('formatSize', () => {
  it('returns "0 B" for 0 bytes', () => {
    expect(formatSize(0)).toBe('0 B');
  });

  it('formats bytes correctly', () => {
    expect(formatSize(512)).toBe('512.0 B');
  });

  it('formats kilobytes correctly', () => {
    expect(formatSize(1024)).toBe('1.0 KB');
  });

  it('formats megabytes correctly', () => {
    expect(formatSize(1048576)).toBe('1.0 MB');
  });

  it('formats gigabytes correctly', () => {
    expect(formatSize(1073741824)).toBe('1.0 GB');
  });

  it('formats terabytes correctly', () => {
    expect(formatSize(1099511627776)).toBe('1.0 TB');
  });

  it('caps at TB', () => {
    const huge = 1024 * 1024 * 1024 * 1024 * 1024;
    expect(formatSize(huge)).toContain('TB');
  });
});

describe('formatTime', () => {
  it('formats a timestamp to YYYY-MM-DD HH:mm:ss', () => {
    const result = formatTime(1700000000);
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$);
  });

  it('handles millisecond timestamps', () => {
    const result = formatTime(1700000000000);
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$);
  });
});

describe('formatRelativeTime', () => {
  it('returns "刚刚" for current time', () => {
    const now = Math.floor(Date.now() / 1000);
    expect(formatRelativeTime(now)).toBe('刚刚');
  });

  it('returns minutes for recent past', () => {
    const fiveMinAgo = Math.floor(Date.now() / 1000) - 300;
    expect(formatRelativeTime(fiveMinAgo)).toBe('5 分钟前');
  });

  it('returns hours for older timestamps', () => {
    const twoHoursAgo = Math.floor(Date.now() / 1000) - 7200;
    expect(formatRelativeTime(twoHoursAgo)).toBe('2 小时前');
  });

  it('returns days for much older timestamps', () => {
    const threeDaysAgo = Math.floor(Date.now() / 1000) - 259200;
    expect(formatRelativeTime(threeDaysAgo)).toBe('3 天前');
  });
});

describe('getFileTypeIcon', () => {
  it('returns video icon for VIDEO', () => {
    expect(getFileTypeIcon('VIDEO')).toBe('🎬');
  });

  it('returns image icon for IMAGE', () => {
    expect(getFileTypeIcon('IMAGE')).toBe('🖼');
  });

  it('returns default for null', () => {
    expect(getFileTypeIcon(null)).toBe('📄');
  });
});

describe('formatPercent', () => {
  it('returns correct percentage', () => {
    expect(formatPercent(50, 200)).toBe('25.0%');
  });

  it('handles zero total', () => {
    expect(formatPercent(0, 0)).toBe('0.0%');
  });

  it('handles 100%', () => {
    expect(formatPercent(100, 100)).toBe('100.0%');
  });
});
