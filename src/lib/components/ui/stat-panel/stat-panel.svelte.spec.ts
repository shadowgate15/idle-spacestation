import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import type { Component } from 'svelte';
import StatPanel from './stat-panel.svelte';

function bodySnippet(innerHtml: string) {
  return createRawSnippet(() => ({
    render: () => `<div data-testid="snippet-body" style="display:contents">${innerHtml}</div>`,
  }));
}

describe('StatPanel', () => {
  it('renders heading inside the <section> as an <h2>', async () => {
    const view = await render(StatPanel as Component, {
      props: {
        heading: 'Resources',
        children: bodySnippet('<dt>Materials</dt><dd>137</dd><div></div>'),
      },
    });

    try {
      const h2 = page.getByText('Resources').element();
      expect(h2.tagName).toBe('H2');
      expect(h2.closest('section')).not.toBeNull();
    } finally {
      await view.unmount();
    }
  });

  it('renders children inside a <dl> with the load-bearing grid classes', async () => {
    const view = await render(StatPanel as Component, {
      props: {
        heading: 'Crew & Power',
        children: bodySnippet('<dt>Crew</dt><dd>3 / 8</dd><div data-testid="bar-slot"></div>'),
      },
    });

    try {
      const child = page.getByTestId('snippet-body').element();
      const dl = child.parentElement;
      expect(dl).not.toBeNull();
      expect(dl!.tagName).toBe('DL');
      expect(dl!.classList.contains('grid')).toBe(true);
      expect(dl!.classList.contains('grid-cols-[auto_1fr_auto]')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('merges class prop onto the outer <section> via cn()', async () => {
    const view = await render(StatPanel as Component, {
      props: {
        heading: 'Survey & Tier',
        class: 'custom-panel-class',
        children: bodySnippet('<dt>X</dt><dd>1</dd><div></div>'),
      },
    });

    try {
      const heading = page.getByText('Survey & Tier').element();
      const section = heading.closest('section');
      expect(section).not.toBeNull();
      expect(section!.classList.contains('custom-panel-class')).toBe(true);
      expect(section!.classList.contains('rounded-lg')).toBe(true);
      expect(section!.classList.contains('border')).toBe(true);
    } finally {
      await view.unmount();
    }
  });

  it('outer element is a <section>', async () => {
    const view = await render(StatPanel as Component, {
      props: {
        heading: 'Panel',
        children: bodySnippet('<dt>A</dt><dd>1</dd><div></div>'),
      },
    });

    try {
      const heading = page.getByText('Panel').element();
      const outer = heading.parentElement;
      expect(outer).not.toBeNull();
      expect(outer!.tagName).toBe('SECTION');
    } finally {
      await view.unmount();
    }
  });

  it('forwards arbitrary attributes (data-testid) onto the <section>', async () => {
    const view = await render(StatPanel as Component, {
      props: {
        heading: 'Forwarded',
        'data-testid': 'panel-forwarded',
        children: bodySnippet('<dt>A</dt><dd>1</dd><div></div>'),
      },
    });

    try {
      const section = page.getByTestId('panel-forwarded').element();
      expect(section.tagName).toBe('SECTION');
    } finally {
      await view.unmount();
    }
  });
});
