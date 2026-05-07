import type { GameSnapshot } from './types';
import { GameStateInitError } from './types';

export type GameStateStatus = 'idle' | 'loading' | 'ready' | 'error';

function createGameState() {
	let snapshot = $state<GameSnapshot | null>(null);
	let status = $state<GameStateStatus>('idle');
	let error = $state<Error | null>(null);

	return {
		get snapshot() {
			return snapshot;
		},
		get status() {
			return status;
		},
		get error() {
			return error;
		},
		async init(): Promise<void> {
			throw new GameStateInitError('not implemented (Task 12)');
		},
		dispose(): void {
			snapshot = null;
			status = 'idle';
			error = null;
		},
		deferUntilBlur(_focused: boolean): void {
			throw new Error('not implemented (Task 13)');
		},
		_setSnapshot(s: GameSnapshot | null): void {
			snapshot = s;
		},
	};
}

export const gameState = createGameState();
