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

  // Header HUD derives directly from the push-based gameState store.
  // Recomputes whenever a new snapshot lands; renders placeholders until ready.
  const snapshot = $derived(gameState.snapshot);
  const overview = $derived(snapshot?.routes.overview ?? null);
  const materialsRate = $derived(
    overview?.resourceDeltas.find((d) => d.id === 'materials')?.deltaPerSecond ?? 0,
  );
  const dataRate = $derived(
    overview?.resourceDeltas.find((d) => d.id === 'data')?.deltaPerSecond ?? 0,
  );
  const powerRate = $derived(
    overview?.resourceDeltas.find((d) => d.id === 'power')?.deltaPerSecond ?? 0,
  );

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

  // Expose gameState + gameGateway on window in fixture mode so E2E tests
  // can drive snapshot updates deterministically (replaces the old
  // periodic-polling assumption now that snapshots are push-based).
  function exposeFixtureGlobals(): void {
    window.__gameState = gameState;
    window.__gameGateway = gameGateway;
  }

  function setupFocusDeferral(): () => void {
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
    return () => {
      document.removeEventListener('focusin', onFocusIn);
      document.removeEventListener('focusout', onFocusOut);
    };
  }

  async function setupDevtoolsVisibility(
    localOverride: boolean,
  ): Promise<() => void | Promise<void>> {
    const isFixtureMode = isFixtureModeEnabled();
    let unlisten: (() => void | Promise<void>) | null = null;

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
      if (!devtoolsDestroyed) {
        devtoolsVisible = localOverride || state.visible;
        if (isFixtureMode) {
          setDevtoolsLocalOverride(devtoolsVisible);
        }
      }
    } catch {
      if (localOverride && !devtoolsDestroyed) {
        devtoolsVisible = true;
        if (isFixtureMode) {
          setDevtoolsLocalOverride(true);
        }
      }
    }

    return () => {
      void unlisten?.();
    };
  }

  onMount(() => {
    devtoolsDestroyed = false;

    void gameState.init().catch((err) => {
      console.error('[layout] gameState.init failed:', err);
    });

    if (isFixtureModeEnabled() && typeof window !== 'undefined') {
      exposeFixtureGlobals();
    }

    const cleanupFocus = setupFocusDeferral();
    const isFixtureMode = isFixtureModeEnabled();

    if (!IS_DEBUG && !isFixtureMode) {
      return () => {
        devtoolsDestroyed = true;
        cleanupFocus();
        gameState.dispose();
      };
    }

    // localOverride is only meaningful in fixture mode (no Tauri backend).
    // In a real Tauri debug session the backend is authoritative; reading
    // localStorage there would let a developer bypass the Debug menu toggle.
    const localOverride =
      isFixtureMode && window.localStorage.getItem(DEVTOOLS_STORAGE_KEY) === 'true';
    let cleanupDevtools: (() => void | Promise<void>) | null = null;

    void setupDevtoolsVisibility(localOverride).then((cleanup) => {
      cleanupDevtools = cleanup;
    });

    return () => {
      devtoolsDestroyed = true;
      cleanupFocus();
      gameState.dispose();
      void cleanupDevtools?.();
    };
  });
</script>

<div data-testid="game-shell" class="flex min-h-screen flex-col bg-background text-foreground">
  <header data-testid="game-header" class="border-b border-border px-6 py-3">
    <div
      data-testid="game-header-hud"
      class="mb-2 flex flex-wrap items-center justify-between gap-x-6 gap-y-2"
    >
      <dl class="flex flex-wrap items-center gap-x-5 gap-y-1 text-sm">
        <div class="flex items-baseline gap-1.5" data-testid="header-stat-materials">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Mat</dt>
          <dd class="font-mono text-foreground tabular-nums">
            {snapshot ? Math.floor(snapshot.resources.materials) : '—'}
          </dd>
          {#if snapshot && materialsRate !== 0}
            <dd
              class="font-mono text-xs tabular-nums {materialsRate > 0
                ? 'text-emerald-400'
                : 'text-rose-400'}"
            >
              {materialsRate > 0 ? '+' : ''}{materialsRate.toFixed(1)}/s
            </dd>
          {/if}
        </div>

        <div class="flex items-baseline gap-1.5" data-testid="header-stat-data">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Data</dt>
          <dd class="font-mono text-foreground tabular-nums">
            {snapshot ? Math.floor(snapshot.resources.data) : '—'}
          </dd>
          {#if snapshot && dataRate !== 0}
            <dd
              class="font-mono text-xs tabular-nums {dataRate > 0
                ? 'text-emerald-400'
                : 'text-rose-400'}"
            >
              {dataRate > 0 ? '+' : ''}{dataRate.toFixed(1)}/s
            </dd>
          {/if}
        </div>

        <div class="flex items-baseline gap-1.5" data-testid="header-stat-crew">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Crew</dt>
          <dd class="font-mono text-foreground tabular-nums">
            {#if snapshot}
              {snapshot.resources.crew.assigned}<span class="text-muted-foreground"
                >/{snapshot.resources.crew.total}</span
              >
            {:else}
              —
            {/if}
          </dd>
        </div>

        <div class="flex items-baseline gap-1.5" data-testid="header-stat-power">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Pwr</dt>
          <dd class="font-mono text-foreground tabular-nums">
            {#if snapshot}
              {snapshot.resources.power.available.toFixed(1)}<span class="text-muted-foreground"
                >/{snapshot.resources.power.generated.toFixed(1)}</span
              >
            {:else}
              —
            {/if}
          </dd>
          {#if snapshot && powerRate !== 0}
            <dd
              class="font-mono text-xs tabular-nums {powerRate > 0
                ? 'text-emerald-400'
                : 'text-rose-400'}"
            >
              {powerRate > 0 ? '+' : ''}{powerRate.toFixed(1)}/s
            </dd>
          {/if}
        </div>
      </dl>

      {#if overview}
        <div class="flex flex-wrap items-center gap-2 text-xs" data-testid="header-context">
          <span
            class="inline-flex items-center rounded border border-border bg-card/50 px-2 py-0.5 font-mono text-muted-foreground"
            data-testid="header-active-planet"
          >
            {overview.activePlanet.name}
          </span>
          <span
            class="inline-flex items-center rounded border border-border bg-card/50 px-2 py-0.5 font-mono text-muted-foreground"
            data-testid="header-station-tier"
          >
            Tier {overview.stationTier.current}/{overview.stationTier.max}
          </span>
        </div>
      {/if}
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
