declare global {
  interface Window {
    __gameState?: ReturnType<typeof import('$lib/game/api/state.svelte').createGameState>;
    __gameGateway?: ReturnType<typeof import('$lib/game/api/gateway.ts').createGameGateway>;
  }
}

export {};
