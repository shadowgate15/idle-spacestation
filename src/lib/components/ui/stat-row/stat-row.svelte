<script lang="ts" module>
  /**
   * @fileoverview
   * StatRow renders a single labelled metric inside a StatPanel's 3-column grid.
   *
   * Layout: `label | value+rate (stacked) | bar`.
   *
   * Empty columns are intentionally blank (no `—` placeholders) to keep
   * alignment scannable across rows in the same panel. The row uses
   * `display: contents` so its three logical cells participate directly in the
   * parent `<dl>` grid tracks.
   */

  export type StatRowSeverity = 'info' | 'warning' | 'critical';

  /**
   * A single row inside a StatPanel. Renders three logical cells:
   * `label | value+rate (stacked) | bar`.
   *
   * The cells participate directly in the parent grid via `display: contents`,
   * so all StatRows in the same panel share column tracks.
   *
   * Picking the right `kind`:
   * - `scalar`   — bare number or string, no rate, no bar (e.g. inventory count)
   * - `stock`    — number with a per-second rate (e.g. Materials with +2.4/s)
   * - `capacity` — current value with a max + bar (e.g. Power 5.2 / 7.0)
   * - `ratio`    — used out of total + bar (e.g. Crew 3 / 8)
   * - `progress` — current toward goal + bar (e.g. Survey 480 / 1000)
   * - `label`    — non-numeric label/value pair, no bar (e.g. Next Target: Mars)
   */
  export type StatRowProps = (
    | {
        /** Bare number or pre-formatted string. */
        kind: 'scalar';
        /** Display value. Numbers are rendered as-is; format upstream if needed. */
        value: number | string;
        /** Optional unit suffix (e.g. 's', '%'). */
        unit?: string;
      }
    | {
        /** Stock value with a rate of change; rate sign drives trend color. */
        kind: 'stock';
        /** Current stockpile (rendered with `toLocaleString`). */
        value: number;
        /** Rate of change per simulated second. Sign drives trend color. */
        perSecond: number;
        /** Optional unit suffix for the value (the rate always shows /s). */
        unit?: string;
      }
    | {
        /** Current value with a maximum capacity; renders `current / max` plus a bar. */
        kind: 'capacity';
        /** Current value (e.g. available power). */
        current: number;
        /** Maximum capacity (e.g. generated power). */
        max: number;
        /** Optional rate of change rendered beneath the value. */
        perSecond?: number;
        /** Optional unit suffix. */
        unit?: string;
        /** Decimal places for current/max display. Default: 0. */
        precision?: number;
      }
    | {
        /** Used out of total; renders `used / total` plus a bar. */
        kind: 'ratio';
        /** Used / assigned count. */
        used: number;
        /** Total / capacity count. */
        total: number;
        /** Optional unit suffix. */
        unit?: string;
      }
    | {
        /** Current progress toward a goal; renders `current / goal` plus a bar. */
        kind: 'progress';
        /** Current progress value. */
        current: number;
        /** Goal value. */
        goal: number;
        /** Optional pre-formatted string used as-is for the current value (e.g. '01:23'). */
        formattedCurrent?: string;
        /** Optional pre-formatted string used as-is for the goal value (e.g. '02:00'). */
        formattedGoal?: string;
      }
    | {
        /** Non-numeric label/value pair (e.g. "Next Target: Mars"). No bar. */
        kind: 'label';
        /** Display value (string). */
        value: string;
      }
  ) & {
    /** Label rendered in col 1 (uppercase, muted). */
    label: string;
    /** Optional severity chip in col 1; tints the value text and bar. */
    severity?: StatRowSeverity;
    /** Optional class merged onto the row wrapper. */
    class?: string;
  };
</script>

<script lang="ts">
  import { cn } from '$lib/utils.js';

  const props: StatRowProps = $props();
  const label = $derived(props.label);
  const severity = $derived(props.severity);
  const className = $derived(props.class);

  function formatRate(perSecond: number): string {
    const sign = perSecond >= 0 ? '+' : '';
    return `${sign}${perSecond.toFixed(1)}/s`;
  }

  function safeFraction(num: number, den: number): number {
    if (den === 0 || !Number.isFinite(num) || !Number.isFinite(den)) return 0;
    return Math.max(0, Math.min(1, num / den));
  }

  const rateText = $derived.by(() => {
    if (props.kind === 'stock') return formatRate(props.perSecond);
    if (props.kind === 'capacity' && typeof props.perSecond === 'number') {
      return formatRate(props.perSecond);
    }
    return '';
  });

  const ratePerSecond = $derived.by(() => {
    if (props.kind === 'stock') return props.perSecond;
    if (props.kind === 'capacity' && typeof props.perSecond === 'number') return props.perSecond;
    return null;
  });

  const rateClass = $derived(
    ratePerSecond === null
      ? 'text-muted-foreground'
      : ratePerSecond > 0
        ? 'text-emerald-400'
        : ratePerSecond < 0
          ? 'text-rose-400'
          : 'text-muted-foreground',
  );

  const valueClass = $derived.by(() => {
    if (severity === 'critical') return 'text-rose-400';
    if (severity === 'warning') return 'text-amber-400';
    if (severity === 'info') return 'text-sky-400';
    if (props.kind === 'stock') {
      if (props.perSecond > 0) return 'text-emerald-400';
      if (props.perSecond < 0) return 'text-rose-400';
    }
    return 'text-foreground';
  });

  const barFillPct = $derived.by(() => {
    if (props.kind === 'capacity') return safeFraction(props.current, props.max) * 100;
    if (props.kind === 'ratio') return safeFraction(props.used, props.total) * 100;
    if (props.kind === 'progress') return safeFraction(props.current, props.goal) * 100;
    return 0;
  });

  const barColorClass = $derived(
    severity === 'critical'
      ? 'bg-rose-500'
      : severity === 'warning'
        ? 'bg-amber-500'
        : 'bg-emerald-500',
  );

  const showBar = $derived(
    props.kind === 'capacity' || props.kind === 'ratio' || props.kind === 'progress',
  );

  const valueText = $derived.by(() => {
    switch (props.kind) {
      case 'scalar':
        return props.unit ? `${props.value} ${props.unit}` : String(props.value);
      case 'stock': {
        const base = props.value.toLocaleString();
        return props.unit ? `${base} ${props.unit}` : base;
      }
      case 'capacity': {
        const p = props.precision ?? 0;
        const base = `${props.current.toFixed(p)} / ${props.max.toFixed(p)}`;
        return props.unit ? `${base} ${props.unit}` : base;
      }
      case 'ratio': {
        const base = `${props.used} / ${props.total}`;
        return props.unit ? `${base} ${props.unit}` : base;
      }
      case 'progress': {
        const cur = props.formattedCurrent ?? String(props.current);
        const goal = props.formattedGoal ?? String(props.goal);
        return `${cur} / ${goal}`;
      }
      case 'label':
        return props.value;
    }
  });
</script>

<div class={cn('contents', className)} data-slot="stat-row" data-severity={severity ?? 'none'}>
  <dt class="flex items-center gap-1.5 text-xs tracking-wide text-muted-foreground uppercase">
    {#if severity === 'critical'}
      <span aria-label="critical" class="text-rose-400">⚠</span>
    {:else if severity === 'warning'}
      <span aria-label="warning" class="text-amber-400">⚡</span>
    {:else if severity === 'info'}
      <span aria-label="info" class="text-sky-400">ℹ</span>
    {/if}
    {label}
  </dt>

  <dd class="flex flex-col leading-tight">
    <span class={cn('text-lg font-bold tabular-nums', valueClass)}>{valueText}</span>
    {#if rateText}
      <span class={cn('text-xs tabular-nums', rateClass)}>{rateText}</span>
    {/if}
  </dd>

  {#if showBar}
    <div class="h-2 w-32 overflow-hidden rounded-full bg-muted" aria-hidden="true">
      <div
        class={cn('h-full rounded-full transition-[width] duration-200', barColorClass)}
        style="width: {barFillPct}%"
      ></div>
    </div>
  {:else}
    <div aria-hidden="true"></div>
  {/if}
</div>
