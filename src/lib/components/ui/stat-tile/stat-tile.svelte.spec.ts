import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { describe, expect, it } from 'vitest';
import type { Component } from 'svelte';
import StatTile from './stat-tile.svelte';

describe('StatTile', () => {
  it('renders <dt>{label}</dt> and <dd>{value}</dd> markup', async () => {
    const view = await mount(StatTile as Component, {
      props: { label: 'Materials', value: 42 },
    });

    try {
      const dt = page.getByText('Materials').element();
      const dd = page.getByText('42').element();
      expect(dt.tagName).toBe('DT');
      expect(dd.tagName).toBe('DD');
    } finally {
      await view.unmount();
    }
  });

  it('default variant: dd has text-foreground class', async () => {
    const view = await mount(StatTile as Component, {
      props: { label: 'Crew', value: '5 / 8' },
    });

    try {
      const dd = page.getByText('5 / 8').element();
      expect(dd.classList.contains('text-foreground')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('variant="positive": dd has text-emerald-400 class', async () => {
    const view = await mount(StatTile as Component, {
      props: { label: 'Net Power', value: '+12.3', variant: 'positive' },
    });

    try {
      const dd = page.getByText('+12.3').element();
      expect(dd.classList.contains('text-emerald-400')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('variant="negative": dd has text-rose-400 class', async () => {
    const view = await mount(StatTile as Component, {
      props: { label: 'Deficit', value: '-3.4', variant: 'negative' },
    });

    try {
      const dd = page.getByText('-3.4').element();
      expect(dd.classList.contains('text-rose-400')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('class prop merges correctly via cn() on the wrapper div', async () => {
    const view = await mount(StatTile as Component, {
      props: { label: 'Data', value: 100, class: 'custom-wrapper-class' },
    });

    try {
      const dt = page.getByText('Data').element();
      const wrapper = dt.parentElement;
      expect(wrapper).not.toBeNull();
      expect(wrapper!.tagName).toBe('DIV');
      expect(wrapper!.classList.contains('custom-wrapper-class')).toBe(true);
      expect(wrapper!.classList.contains('flex')).toBe(true);
      expect(wrapper!.classList.contains('flex-col')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('label has uppercase tracking-wide muted classes', async () => {
    const view = await mount(StatTile as Component, {
      props: { label: 'Power', value: '7.0' },
    });

    try {
      const dt = page.getByText('Power').element();
      expect(dt.classList.contains('text-xs')).toBe(true);
      expect(dt.classList.contains('uppercase')).toBe(true);
      expect(dt.classList.contains('text-muted-foreground')).toBe(true);
    } finally {
      await view.unmount();
    }
  });
});
