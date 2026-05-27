import { describe, it, expect } from 'vitest';
import { getNodeColor, TYPE_COLORS } from '../colors';

describe('getNodeColor', () => {
  it('returns directory color for directories', () => {
    expect(getNodeColor('mydir', true)).toBe('#5b6abf');
  });

  it('returns correct color for known extensions', () => {
    expect(getNodeColor('photo.png', false)).toBe('#2ecc71');
    expect(getNodeColor('video.mp4', false)).toBe('#3498db');
    expect(getNodeColor('archive.zip', false)).toBe('#f39c12');
    expect(getNodeColor('doc.pdf', false)).toBe('#e67e22');
  });

  it('returns default color for unknown extensions', () => {
    expect(getNodeColor('unknown.xyz', false)).toBe('#7f8c8d');
  });

  it('is case sensitive (current behavior)', () => {
    expect(getNodeColor('FILE.PNG', false)).toBe('#2ecc71');
  });
});

describe('TYPE_COLORS', () => {
  it('contains entries for common file types', () => {
    expect(TYPE_COLORS['.png']).toBeDefined();
    expect(TYPE_COLORS['.mp4']).toBeDefined();
    expect(TYPE_COLORS['.zip']).toBeDefined();
    expect(TYPE_COLORS['.pdf']).toBeDefined();
    expect(TYPE_COLORS['.exe']).toBeDefined();
  });
});
