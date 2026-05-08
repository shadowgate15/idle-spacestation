import type {
  GameSnapshot,
  GameViewModels,
  OverviewViewModel,
  PlanetsViewModel,
  PrestigeViewModel,
  RawGameSnapshot,
  ServicesViewModel,
  SystemsViewModel,
} from './types';

const ROUTE_KEYS = ['overview', 'systems', 'services', 'planets', 'prestige'] as const;

export function adaptGameSnapshot(snapshot: RawGameSnapshot): GameSnapshot {
  const cloned = structuredClone(snapshot);
  const routes = Object.fromEntries(ROUTE_KEYS.map((k) => [k, cloned.routeSnapshots[k]])) as unknown as GameViewModels;
  return { ...cloned, routes };
}

export const adaptOverviewViewModel = (s: RawGameSnapshot): OverviewViewModel => adaptGameSnapshot(s).routes.overview;
export const adaptSystemsViewModel = (s: RawGameSnapshot): SystemsViewModel => adaptGameSnapshot(s).routes.systems;
export const adaptServicesViewModel = (s: RawGameSnapshot): ServicesViewModel => adaptGameSnapshot(s).routes.services;
export const adaptPlanetsViewModel = (s: RawGameSnapshot): PlanetsViewModel => adaptGameSnapshot(s).routes.planets;
export const adaptPrestigeViewModel = (s: RawGameSnapshot): PrestigeViewModel => adaptGameSnapshot(s).routes.prestige;
