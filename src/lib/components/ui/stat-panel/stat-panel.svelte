<script lang="ts" module>
  /**
   * @fileoverview
   * StatPanel is a grouping wrapper for related stats. It renders a heading and
   * a 3-column grid (label | value+rate | bar) so child StatRows share column
   * tracks and stay visually aligned.
   */

  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  export type StatPanelProps = {
    /** Heading text displayed at the top of the panel. */
    heading: string;
    /** Optional class merged onto the outer <section>. */
    class?: string;
    /** Renders inside a 3-column grid: label | value+rate | bar. */
    children?: Snippet;
  } & Omit<HTMLAttributes<HTMLElement>, 'class' | 'children'>;
</script>

<script lang="ts">
  import { cn } from '$lib/utils.js';

  let { heading, class: className, children, ...restProps }: StatPanelProps = $props();
</script>

<section class={cn('rounded-lg border border-border bg-card p-4', className)} {...restProps}>
  <h2 class="mb-3 text-base font-semibold text-foreground">{heading}</h2>
  <dl class="grid grid-cols-[auto_1fr_auto] items-center gap-x-6 gap-y-3">
    {@render children?.()}
  </dl>
</section>
