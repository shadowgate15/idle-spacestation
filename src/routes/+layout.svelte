<script lang="ts">
  import './layout.css';
  import { onMount, type Snippet } from 'svelte';
  import DevtoolsOverlay from '$lib/components/DevtoolsOverlay.svelte';
  import { gameGateway } from '$lib/game/api';
  import { gameState } from '$lib/game/api/state.svelte';

  const IS_DEBUG = import.meta.env.DEV;
  const DEVTOOLS_STORAGE_KEY = 'idle-spacestation.devtools-open';
  const E2E_FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';

  let { children }: { children: Snippet } = $props();
  let devtoolsVisible = $state(false);
  let devtoolsDestroyed = false;

  function isFixtureModeEnabled() {
    if (typeof window === 'undefined') {
      return false;
    }

    return window.localStorage.getItem(E2E_FIXTURE_STORAGE_KEY) !== null;
  }

  function isEditableDevtoolsInputFocused(): boolean {
    if (typeof document === 'undefined') return false;
    const active = document.activeElement;
    if (
      !(
        active instanceof HTMLInputElement ||
        active instanceof HTMLTextAreaElement ||
        active instanceof HTMLSelectElement
      )
    )
      return false;
    return Boolean(active.closest('[data-testid="devtools-overlay"]'));
  }

  function setDevtoolsLocalOverride(visible: boolean) {
    if (typeof window === 'undefined') return;

    if (visible) {
      window.localStorage.setItem(DEVTOOLS_STORAGE_KEY, 'true');
      return;
    }

    window.localStorage.removeItem(DEVTOOLS_STORAGE_KEY);
  }

  async function handleDevtoolsClose() {
    devtoolsVisible = false;
    if (isFixtureModeEnabled()) {
      setDevtoolsLocalOverride(false);
    }

    try {
      await gameGateway.setDevtoolsVisibility({ visible: false });
    } catch {
      // ignore
    }
  }

  onMount(() => {
    devtoolsDestroyed = false;

    void gameState.init().catch((err) => {
      console.error('[layout] gameState.init failed:', err);
    });

    // Expose gameState + gameGateway on window in fixture mode so E2E tests
    // can drive snapshot updates deterministically (replaces the old
    // periodic-polling assumption now that snapshots are push-based).
    if (isFixtureModeEnabled() && typeof window !== 'undefined') {
      (window as unknown as Record<string, unknown>).__gameState = gameState;
      (window as unknown as Record<string, unknown>).__gameGateway = gameGateway;
    }

    function onFocusIn() {
      if (isEditableDevtoolsInputFocused()) {
        gameState.deferUntilBlur(true);
      }
    }
    function onFocusOut() {
      if (!isEditableDevtoolsInputFocused()) {
        gameState.deferUntilBlur(false);
      }
    }
    document.addEventListener('focusin', onFocusIn);
    document.addEventListener('focusout', onFocusOut);

    const isFixtureMode = isFixtureModeEnabled();

    if (!IS_DEBUG && !isFixtureMode) {
      return () => {
        devtoolsDestroyed = true;
        document.removeEventListener('focusin', onFocusIn);
        document.removeEventListener('focusout', onFocusOut);
        gameState.dispose();
      };
    }

    // localOverride is only meaningful in fixture mode (no Tauri backend).
    // In a real Tauri debug session the backend is authoritative; reading
    // localStorage there would let a developer bypass the Debug menu toggle.
    const localOverride =
      isFixtureMode && window.localStorage.getItem(DEVTOOLS_STORAGE_KEY) === 'true';
    let unlisten: (() => void | Promise<void>) | null = null;

    const initialize = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<{ visible: boolean }>('devtools:visibility-changed', (event) => {
          if (devtoolsDestroyed) return;

          devtoolsVisible = event.payload.visible;
          // Only persist visibility to localStorage in fixture mode so that
          // page reloads during E2E tests restore the overlay state.
          if (isFixtureMode) {
            setDevtoolsLocalOverride(event.payload.visible);
          }
        });
      } catch {
        // not in Tauri
      }

      try {
        const state = await gameGateway.getDevtoolsState();
        if (devtoolsDestroyed) return;

        devtoolsVisible = localOverride || state.visible;
        if (isFixtureMode) {
          setDevtoolsLocalOverride(devtoolsVisible);
        }
      } catch {
        if (localOverride && !devtoolsDestroyed) {
          devtoolsVisible = true;
          if (isFixtureMode) {
            setDevtoolsLocalOverride(true);
          }
        }
      }
    };

    void initialize();

    return () => {
      devtoolsDestroyed = true;
      document.removeEventListener('focusin', onFocusIn);
      document.removeEventListener('focusout', onFocusOut);
      gameState.dispose();
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
    <DevtoolsOverlay
      snapshot={gameState.snapshot}
      gateway={gameGateway}
      onClose={handleDevtoolsClose}
    />
  {/if}
</div>
