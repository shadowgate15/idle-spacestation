import { describe, it, expect } from 'vitest';
import { isInRange, isAtLeast } from './utils';

describe('isInRange', () => {
  it('returns true for a number within range (inclusive)', () => {
    expect(isInRange(5, 0, 10)).toBe(true);
  });

  it('returns true for a number at min boundary', () => {
    expect(isInRange(0, 0, 10)).toBe(true);
  });

  it('returns true for a number at max boundary', () => {
    expect(isInRange(10, 0, 10)).toBe(true);
  });

  it('returns false for a number below min', () => {
    expect(isInRange(-1, 0, 10)).toBe(false);
  });

  it('returns false for a number above max', () => {
    expect(isInRange(11, 0, 10)).toBe(false);
  });

  it('returns false for undefined', () => {
    expect(isInRange(undefined, 0, 10)).toBe(false);
  });

  it('returns false for NaN', () => {
    expect(isInRange(NaN, 0, 10)).toBe(false);
  });

  it('acts as a type guard (TypeScript narrowing)', () => {
    const v: number | undefined = 5;
    if (isInRange(v, 0, 10)) {
      // TypeScript should narrow v to number here
      const _n: number = v;
      expect(_n).toBe(5);
    }
  });
});

describe('isAtLeast', () => {
  it('returns true for a number >= min', () => {
    expect(isAtLeast(5, 0)).toBe(true);
  });

  it('returns true for a number at min boundary', () => {
    expect(isAtLeast(0, 0)).toBe(true);
  });

  it('returns true for a very large number', () => {
    expect(isAtLeast(1000, 0)).toBe(true);
  });

  it('returns false for a number below min', () => {
    expect(isAtLeast(-1, 0)).toBe(false);
  });

  it('returns false for undefined', () => {
    expect(isAtLeast(undefined, 0)).toBe(false);
  });

  it('returns false for NaN', () => {
    expect(isAtLeast(NaN, 0)).toBe(false);
  });

  it('acts as a type guard (TypeScript narrowing)', () => {
    const v: number | undefined = 10;
    if (isAtLeast(v, 5)) {
      // TypeScript should narrow v to number here
      const _n: number = v;
      expect(_n).toBe(10);
    }
  });
});
