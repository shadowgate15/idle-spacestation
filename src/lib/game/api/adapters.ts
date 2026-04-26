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

export function adaptOverviewViewModel(snapshot: RawGameSnapshot): OverviewViewModel {
  return structuredClone(snapshot.routeSnapshots.overview);
}

export function adaptSystemsViewModel(snapshot: RawGameSnapshot): SystemsViewModel {
  return structuredClone(snapshot.routeSnapshots.systems);
}

export function adaptServicesViewModel(snapshot: RawGameSnapshot): ServicesViewModel {
  return structuredClone(snapshot.routeSnapshots.services);
}

export function adaptPlanetsViewModel(snapshot: RawGameSnapshot): PlanetsViewModel {
  return structuredClone(snapshot.routeSnapshots.planets);
}

export function adaptPrestigeViewModel(snapshot: RawGameSnapshot): PrestigeViewModel {
  return structuredClone(snapshot.routeSnapshots.prestige);
}

export function adaptGameViewModels(snapshot: RawGameSnapshot): GameViewModels {
  return {
    overview: adaptOverviewViewModel(snapshot),
    systems: adaptSystemsViewModel(snapshot),
    services: adaptServicesViewModel(snapshot),
    planets: adaptPlanetsViewModel(snapshot),
    prestige: adaptPrestigeViewModel(snapshot),
  };
}

export function adaptGameSnapshot(snapshot: RawGameSnapshot): GameSnapshot {
  return {
    ...structuredClone(snapshot),
    routes: adaptGameViewModels(snapshot),
  };
}
