<script lang="ts">
  import './layout.css';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div data-testid="game-shell" class="flex min-h-screen flex-col bg-background text-foreground">
  <header data-testid="game-header" class="border-b border-border px-6 py-4">
    <div class="mb-3 flex flex-col gap-1">
      <p class="text-xs tracking-widest text-muted-foreground uppercase">
        Orbital Operations Console
      </p>
      <h1 class="text-xl font-bold tracking-tight text-foreground">Idle Space Station</h1>
      <p class="text-sm text-muted-foreground">
        Dark-only tactical interface for steady orbital expansion.
      </p>
    </div>
    <div class="mb-3 flex items-center gap-2">
      <span
        class="inline-flex items-center rounded border border-border px-2 py-0.5 font-mono text-xs text-muted-foreground"
        >Dock 03 // Dark Watch</span
      >
      <span
        class="inline-flex items-center rounded border border-border px-2 py-0.5 font-mono text-xs text-muted-foreground"
        >Beacon #0984e3</span
      >
    </div>
    <nav class="flex gap-4">
      <a
        href="#overview-panel"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Overview</a
      >
      <a
        href="#systems-panel"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Systems</a
      >
      <a
        href="#alerts-panel"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Alerts</a
      >
    </nav>
  </header>
  <main class="flex-1 px-6 py-6">
    {@render children()}
  </main>
</div>
