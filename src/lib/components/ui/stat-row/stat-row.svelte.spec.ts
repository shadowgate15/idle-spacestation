import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import { describe, expect, it } from 'vitest';
import type { Component } from 'svelte';
import StatRow from './stat-row.svelte';

describe('StatRow', () => {
  it('kind="scalar" renders bare value inside a <dd>', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'scalar', label: 'Materials', value: 137 },
    });

    try {
      const value = page.getByText('137').element();
      expect(value.closest('dd')).not.toBeNull();
    } finally {
      await view.unmount();
    }
  });

  it('kind="scalar" with unit appends the unit', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'scalar', label: 'Storage', value: 137, unit: 'GB' },
    });

    try {
      await expect.element(page.getByText('137 GB')).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('kind="stock" with positive perSecond renders value + "+2.4/s" rate in emerald', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'stock', label: 'Materials', value: 137, perSecond: 2.4 },
    });

    try {
      await expect.element(page.getByText('137')).toBeInTheDocument();
      const rate = page.getByText('+2.4/s').element();
      expect(rate.classList.contains('text-emerald-400')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('kind="stock" with negative perSecond renders "-1.5/s" in rose', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'stock', label: 'Power', value: 5, perSecond: -1.5 },
    });

    try {
      const rate = page.getByText('-1.5/s').element();
      expect(rate.classList.contains('text-rose-400')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('kind="stock" with zero perSecond renders "+0.0/s" in muted', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'stock', label: 'Idle', value: 0, perSecond: 0 },
    });

    try {
      const rate = page.getByText('+0.0/s').element();
      expect(rate.classList.contains('text-muted-foreground')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('kind="capacity" renders "current / max" with bar at correct fill percent', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'capacity', label: 'Power', current: 5.2, max: 7.0, precision: 1 },
    });

    try {
      await expect.element(page.getByText('5.2 / 7.0')).toBeInTheDocument();
      const dt = page.getByText('Power').element();
      const wrapper = dt.parentElement;
      const bar = wrapper!.querySelector('[aria-hidden="true"].bg-muted > div') as HTMLElement;
      expect(bar).not.toBeNull();
      const widthStr = bar.style.width;
      const widthPct = Number.parseFloat(widthStr);
      expect(widthPct).toBeGreaterThan(73);
      expect(widthPct).toBeLessThan(75);
    } finally {
      await view.unmount();
    }
  });

  it('kind="ratio" renders "used / total" with bar at ~37.5%', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'ratio', label: 'Crew', used: 3, total: 8 },
    });

    try {
      await expect.element(page.getByText('3 / 8')).toBeInTheDocument();
      const dt = page.getByText('Crew').element();
      const wrapper = dt.parentElement;
      const bar = wrapper!.querySelector('[aria-hidden="true"].bg-muted > div') as HTMLElement;
      const widthPct = Number.parseFloat(bar.style.width);
      expect(widthPct).toBeCloseTo(37.5, 1);
    } finally {
      await view.unmount();
    }
  });

  it('kind="ratio" with used=0 total=0 does not NaN; bar at 0%', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'ratio', label: 'Empty', used: 0, total: 0 },
    });

    try {
      await expect.element(page.getByText('0 / 0')).toBeInTheDocument();
      const dt = page.getByText('Empty').element();
      const wrapper = dt.parentElement;
      const bar = wrapper!.querySelector('[aria-hidden="true"].bg-muted > div') as HTMLElement;
      expect(bar.style.width).toBe('0%');
    } finally {
      await view.unmount();
    }
  });

  it('kind="progress" uses formattedCurrent / formattedGoal when provided', async () => {
    const view = await render(StatRow as Component, {
      props: {
        kind: 'progress',
        label: 'Stable Power Time',
        current: 83,
        goal: 120,
        formattedCurrent: '01:23',
        formattedGoal: '02:00',
      },
    });

    try {
      await expect.element(page.getByText('01:23 / 02:00')).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('kind="label" renders only label + value, no bar', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'label', label: 'Next Target', value: 'Mars' },
    });

    try {
      await expect.element(page.getByText('Mars')).toBeInTheDocument();
      const dt = page.getByText('Next Target').element();
      const wrapper = dt.parentElement;
      const bar = wrapper!.querySelector('.bg-muted');
      expect(bar).toBeNull();
    } finally {
      await view.unmount();
    }
  });

  it('severity="critical" renders the ⚠ icon in the label cell', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'scalar', label: 'Critical', value: 0, severity: 'critical' },
    });

    try {
      const icon = page.getByLabelText('critical').element();
      expect(icon.textContent).toContain('⚠');
      expect(icon.classList.contains('text-rose-400')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('severity="warning" tints the bar amber', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'capacity', label: 'Power', current: 6.5, max: 7.0, severity: 'warning' },
    });

    try {
      const dt = page.getByText('Power').element();
      const wrapper = dt.parentElement;
      const bar = wrapper!.querySelector('[aria-hidden="true"].bg-muted > div') as HTMLElement;
      expect(bar.classList.contains('bg-amber-500')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('class prop merges via cn() onto the row wrapper', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'scalar', label: 'X', value: 1, class: 'custom-row-class' },
    });

    try {
      const dt = page.getByText('X').element();
      const wrapper = dt.parentElement;
      expect(wrapper).not.toBeNull();
      expect(wrapper!.classList.contains('custom-row-class')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('wrapper uses the contents class so it participates in parent grid', async () => {
    const view = await render(StatRow as Component, {
      props: { kind: 'scalar', label: 'Y', value: 2 },
    });

    try {
      const dt = page.getByText('Y').element();
      const wrapper = dt.parentElement;
      expect(wrapper).not.toBeNull();
      expect(wrapper!.classList.contains('contents')).toBe(true);
    } finally {
      await view.unmount();
    }
  });
});
