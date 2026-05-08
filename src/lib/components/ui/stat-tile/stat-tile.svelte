<script lang="ts" module>
  export type StatTileVariant = 'default' | 'positive' | 'negative';

  export type StatTileProps = {
    label: string;
    value: string | number;
    variant?: StatTileVariant;
    class?: string;
  };

  const VALUE_VARIANT_CLASS: Record<StatTileVariant, string> = {
    default: 'text-foreground',
    positive: 'text-emerald-400',
    negative: 'text-rose-400',
  };
</script>

<script lang="ts">
  import { cn } from '$lib/utils.js';

  let {
    label,
    value,
    variant = 'default',
    class: className,
  }: StatTileProps = $props();

  const valueClass = $derived(VALUE_VARIANT_CLASS[variant]);
</script>

<div class={cn('flex flex-col', className)}>
  <dt class="text-xs tracking-wide text-muted-foreground uppercase">{label}</dt>
  <dd class={cn('text-lg font-bold', valueClass)}>{value}</dd>
</div>
