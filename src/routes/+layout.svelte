<script lang="ts">
  import './layout.css';
  import { onMount, type Snippet } from 'svelte';
  import DevtoolsOverlay from '$lib/components/DevtoolsOverlay.svelte';
  import { gameGateway } from '$lib/game/api';
  import type { GameSnapshot } from '$lib/game/api/types';

  const IS_DEBUG = import.meta.env.DEV;
  const DEVTOOLS_STORAGE_KEY = 'idle-spacestation.devtools-open';

  let { children }: { children: Snippet } = $props();
  let devtoolsVisible = $state(false);
  let devtoolsSnapshot = $state<GameSnapshot | null>(null);
  let devtoolsPollInterval: ReturnType<typeof setInterval> | null = null;
  let devtoolsDestroyed = false;

  function setDevtoolsLocalOverride(visible: boolean) {
    if (typeof window === 'undefined') return;

    if (visible) {
      window.localStorage.setItem(DEVTOOLS_STORAGE_KEY, 'true');
      return;
    }

    window.localStorage.removeItem(DEVTOOLS_STORAGE_KEY);
  }

  function stopDevtoolsPolling() {
    if (devtoolsPollInterval) {
      clearInterval(devtoolsPollInterval);
      devtoolsPollInterval = null;
    }
  }

  async function refreshDevtoolsSnapshot() {
    try {
      const state = await gameGateway.getDevtoolsState();
      if (!devtoolsDestroyed) {
        devtoolsSnapshot = state.snapshot;
      }
    } catch {
      // not available
    }
  }

  function startDevtoolsPolling() {
    stopDevtoolsPolling();
    devtoolsPollInterval = setInterval(async () => {
      if (!devtoolsVisible || devtoolsDestroyed) {
        stopDevtoolsPolling();
        return;
      }

      await refreshDevtoolsSnapshot();
    }, 1000);
  }

  async function handleDevtoolsClose() {
    devtoolsVisible = false;
    devtoolsSnapshot = null;
    setDevtoolsLocalOverride(false);
    stopDevtoolsPolling();

    try {
      await gameGateway.setDevtoolsVisibility({ visible: false });
    } catch {
      // ignore
    }
  }

  $effect(() => {
    if (!IS_DEBUG || devtoolsDestroyed) {
      stopDevtoolsPolling();
      return;
    }

    if (devtoolsVisible) {
      startDevtoolsPolling();
      return;
    }

    stopDevtoolsPolling();
  });

  onMount(() => {
    devtoolsDestroyed = false;

    if (!IS_DEBUG) {
      return () => {
        devtoolsDestroyed = true;
        stopDevtoolsPolling();
      };
    }

    const localOverride = window.localStorage.getItem(DEVTOOLS_STORAGE_KEY) === 'true';
    let unlisten: (() => void | Promise<void>) | null = null;

    const initialize = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<{ visible: boolean }>('devtools:visibility-changed', (event) => {
          if (devtoolsDestroyed) return;

          devtoolsVisible = event.payload.visible;
          setDevtoolsLocalOverride(event.payload.visible);

          if (event.payload.visible) {
            void refreshDevtoolsSnapshot();
            return;
          }

          devtoolsSnapshot = null;
          stopDevtoolsPolling();
        });
      } catch {
        // not in Tauri
      }

      try {
        const state = await gameGateway.getDevtoolsState();
        if (devtoolsDestroyed) return;

        devtoolsVisible = localOverride || state.visible;
        setDevtoolsLocalOverride(devtoolsVisible);

        if (devtoolsVisible) {
          devtoolsSnapshot = state.snapshot;
        }
      } catch {
        if (localOverride && !devtoolsDestroyed) {
          devtoolsVisible = true;
          setDevtoolsLocalOverride(true);
          await refreshDevtoolsSnapshot();
        }
      }
    };

    void initialize();

    return () => {
      devtoolsDestroyed = true;
      stopDevtoolsPolling();
      void unlisten?.();
    };
  });
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
      <a href="/" class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        >Overview</a
      >
      <a
        href="/systems"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Systems</a
      >
      <a
        href="/services"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Services</a
      >
      <a
        href="/planets"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Planets</a
      >
      <a
        href="/prestige"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground">Prestige</a
      >
    </nav>
  </header>
  <main class="flex-1 px-6 py-6">
    {@render children()}
  </main>

  {#if devtoolsVisible}
    <DevtoolsOverlay snapshot={devtoolsSnapshot} gateway={gameGateway} onClose={handleDevtoolsClose} />
  {/if}
</div>
