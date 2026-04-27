export const previewFixtureNames = ['starter', 'deficit', 'all-planets', 'prestige-ready'] as const;

export type PreviewFixtureName = (typeof previewFixtureNames)[number];

export type PlanetId = 'solstice-anchor' | 'cinder-forge' | 'aurora-pier';

export type SystemId = 'reactor-core' | 'habitat-ring' | 'logistics-spine' | 'survey-array';

export type ServiceId =
  | 'solar-harvester'
  | 'ore-reclaimer'
  | 'survey-uplink'
  | 'maintenance-bay'
  | 'command-relay'
  | 'fabrication-loop';

export type DoctrineId =
  | 'efficient-shifts'
  | 'deep-survey-protocols'
  | 'hardened-relays'
  | 'frontier-charters';

export type ServiceFamily = 'production' | 'support' | 'conversion';

export type PlanetModifierTarget =
  | 'crew-efficiency'
  | 'data-output'
  | 'materials-output'
  | 'service-power-upkeep'
  | 'crew-capacity';

export type ResourceId = 'power' | 'materials' | 'data' | 'crew';

export type GuidanceTrigger =
  | 'review-station-status'
  | 'upgrade-reactor-core'
  | 'upgrade-logistics-spine'
  | 'activate-survey-uplink'
  | 'clear-power-deficit'
  | 'discover-second-planet'
  | 'spend-doctrine-fragment'
  | 'prestige-now';

export type WarningSeverity = 'warning' | 'critical' | 'info';

export type ServiceStatus = 'active' | 'paused' | 'disabled';

export type ServiceDisabledReasonCode = 'capacity' | 'crew' | 'deficit' | 'power-cap';

export type PrestigeReasonCode =
  | 'station-tier-below-four'
  | 'needs-two-non-starter-planets'
  | 'unstable-net-power';

export type SystemUpgradeRejectionCode = 'unknown-system' | 'max-level' | 'insufficient-materials';

export type ServiceActivationRejectionCode =
  | 'unknown-service'
  | 'capacity-reached'
  | 'insufficient-crew'
  | 'power-deficit';

export type ServiceCrewAssignmentRejectionCode =
  | 'unknown-service'
  | 'invalid-assignment'
  | 'insufficient-crew';

export type ServicePriorityRejectionCode = 'unknown-service' | 'priority-limit';

export type SurveyStartRejectionCode = 'all-planets-discovered';

export type DoctrinePurchaseRejectionCode =
  | 'unknown-doctrine'
  | 'already-unlocked'
  | 'insufficient-fragments';

export type PrestigeRejectionCode = PrestigeReasonCode | 'confirmation-required';

export interface PowerSnapshot {
  generated: number;
  reserved: number;
  available: number;
}

export interface CrewSnapshot {
  total: number;
  assigned: number;
  available: number;
}

export interface PlanetModifierSnapshot {
  target: PlanetModifierTarget;
  label: string;
  percent: number;
  effectText: string;
}

export interface ResourceDeltaSnapshot {
  id: ResourceId;
  label: string;
  deltaPerSecond: number;
  trend: 'positive' | 'neutral' | 'negative';
}

export interface WarningSnapshot {
  code: string;
  severity: WarningSeverity;
  title: string;
  body: string;
}

export interface StationTierSnapshot {
  current: number;
  max: number;
  label: string;
}

export interface ServiceUtilizationSnapshot {
  active: number;
  capacity: number;
  available: number;
  summary: string;
}

export interface SurveyProgressSnapshot {
  current: number;
  nextThreshold: number | null;
  nextPlanetId: PlanetId | null;
  nextPlanetName: string | null;
  summary: string;
}

export interface OverviewRouteSnapshot {
  activePlanet: {
    id: PlanetId;
    name: string;
    description: string;
    modifiers: PlanetModifierSnapshot[];
  };
  resourceDeltas: ResourceDeltaSnapshot[];
  deficitWarnings: WarningSnapshot[];
  stationTier: StationTierSnapshot;
  serviceUtilization: ServiceUtilizationSnapshot;
  surveyProgress: SurveyProgressSnapshot;
  guidanceTriggers: GuidanceTrigger[];
}

export interface SystemCapSnapshot {
  key: string;
  label: string;
  value: number;
  unit: string;
}

export interface SystemRouteEntrySnapshot {
  id: SystemId;
  name: string;
  description: string;
  level: number;
  maxLevel: number;
  capValues: SystemCapSnapshot[];
  upgradeCostMaterials: number | null;
  canUpgrade: boolean;
  upgradeBlockedReason: string | null;
}

export interface SystemsRouteSnapshot {
  systems: SystemRouteEntrySnapshot[];
}

export interface ServiceCrewAssignmentSnapshot {
  current: number;
  required: number;
}

export interface ServicePowerUsageSnapshot {
  upkeep: number;
  output: number;
}

export interface ServiceRouteEntrySnapshot {
  id: ServiceId;
  name: string;
  description: string;
  family: ServiceFamily;
  priorityOrder: number;
  status: ServiceStatus;
  statusLabel: string;
  desiredActive: boolean;
  crewAssignment: ServiceCrewAssignmentSnapshot;
  powerUsage: ServicePowerUsageSnapshot;
  disabledReasons: ServiceDisabledReasonCode[];
}

export interface ServicesRouteSnapshot {
  services: ServiceRouteEntrySnapshot[];
  utilization: ServiceUtilizationSnapshot;
  deficitWarnings: WarningSnapshot[];
}

export interface PlanetRouteEntrySnapshot {
  id: PlanetId;
  name: string;
  description: string;
  discovered: boolean;
  active: boolean;
  selectableForNextRun: boolean;
  selectabilityReason: string | null;
  modifiers: PlanetModifierSnapshot[];
  surveyThreshold: number | null;
  surveyProgress: number;
}

export interface PlanetsRouteSnapshot {
  activePlanetId: PlanetId;
  planets: PlanetRouteEntrySnapshot[];
  surveyProgress: SurveyProgressSnapshot;
}

export interface DoctrineSnapshot {
  id: DoctrineId;
  name: string;
  description: string;
}

export interface DoctrinePurchaseOptionSnapshot extends DoctrineSnapshot {
  costFragments: number;
  available: boolean;
  blockedReason: DoctrinePurchaseRejectionCode | null;
}

export interface ResetConsequenceSnapshot {
  label: string;
  outcome: 'retain' | 'reset';
  summary: string;
}

export interface PrestigeRouteSnapshot {
  eligibility: {
    eligible: boolean;
    reasonCodes: PrestigeReasonCode[];
    summary: string;
    stablePowerSeconds: number;
    requiredStablePowerSeconds: number;
  };
  doctrineFragments: number;
  unlockedDoctrines: DoctrineSnapshot[];
  purchaseOptions: DoctrinePurchaseOptionSnapshot[];
  resetConsequences: ResetConsequenceSnapshot[];
}

export interface RawSystemStateSnapshot {
  id: SystemId;
  level: number;
}

export interface RawServiceStateSnapshot {
  id: ServiceId;
  desiredActive: boolean;
  isActive: boolean;
  isPaused: boolean;
  pauseReason: ServiceDisabledReasonCode | null;
  priority: number;
  assignedCrew: number;
}

export interface PreviewFixtureState {
  tickCount: number;
  stablePowerSeconds: number;
  activePlanetId: PlanetId;
  discoveredPlanetIds: PlanetId[];
  doctrineIds: DoctrineId[];
  doctrineFragments: number;
  surveyProgress: number;
  materials: number;
  data: number;
  power: PowerSnapshot;
  crew: CrewSnapshot;
  systems: RawSystemStateSnapshot[];
  services: RawServiceStateSnapshot[];
}

export interface RawGameSnapshot {
  meta: {
    source: 'tauri' | 'preview-fixture';
    fixtureName: PreviewFixtureName | null;
    tickCount: number;
  };
  run: {
    activePlanetId: PlanetId;
    discoveredPlanetIds: PlanetId[];
    doctrineIds: DoctrineId[];
    doctrineFragments: number;
    surveyProgress: number;
    stationTier: number;
    stablePowerSeconds: number;
  };
  resources: {
    power: PowerSnapshot;
    materials: number;
    data: number;
    crew: CrewSnapshot;
  };
  systems: RawSystemStateSnapshot[];
  services: RawServiceStateSnapshot[];
  routeSnapshots: {
    overview: OverviewRouteSnapshot;
    systems: SystemsRouteSnapshot;
    services: ServicesRouteSnapshot;
    planets: PlanetsRouteSnapshot;
    prestige: PrestigeRouteSnapshot;
  };
}

export interface GameSnapshot extends RawGameSnapshot {
  routes: RawGameSnapshot['routeSnapshots'];
}

export type OverviewViewModel = OverviewRouteSnapshot;
export type SystemsViewModel = SystemsRouteSnapshot;
export type ServicesViewModel = ServicesRouteSnapshot;
export type PlanetsViewModel = PlanetsRouteSnapshot;
export type PrestigeViewModel = PrestigeRouteSnapshot;

export interface GameViewModels {
  overview: OverviewViewModel;
  systems: SystemsViewModel;
  services: ServicesViewModel;
  planets: PlanetsViewModel;
  prestige: PrestigeViewModel;
}

export interface UpgradeSystemInput {
  systemId: SystemId;
}

export interface SetServiceActivationInput {
  serviceId: ServiceId;
  active: boolean;
}

export interface AssignServiceCrewInput {
  serviceId: ServiceId;
  assignedCrew: number;
}

export interface ReprioritizeServiceInput {
  serviceId: ServiceId;
  direction: 'up' | 'down';
}

export interface PurchaseDoctrineInput {
  doctrineId: DoctrineId;
}

export interface ConfirmPrestigeInput {
  confirm: true;
}

export interface GameActionSuccess {
  ok: true;
  snapshot: RawGameSnapshot;
}

export interface GameActionFailure<TReasonCode extends string> {
  ok: false;
  reasonCode: TReasonCode;
  snapshot: RawGameSnapshot;
}

export type GameActionResponse<TReasonCode extends string> =
  | GameActionSuccess
  | GameActionFailure<TReasonCode>;

export interface GatewayActionSuccess {
  ok: true;
  snapshot: GameSnapshot;
}

export interface GatewayActionFailure<TReasonCode extends string> {
  ok: false;
  reasonCode: TReasonCode;
  snapshot: GameSnapshot;
}

export type GatewayActionResponse<TReasonCode extends string> =
  | GatewayActionSuccess
  | GatewayActionFailure<TReasonCode>;

export interface SaveGameResponse {
  ok: true;
  status: 'saved';
  snapshot: RawGameSnapshot;
}

export interface LoadGameResponse {
  ok: true;
  status: 'loaded';
  snapshot: RawGameSnapshot;
}

export interface GatewaySaveGameResponse {
  ok: true;
  status: 'saved';
  snapshot: GameSnapshot;
}

export interface GatewayLoadGameResponse {
  ok: true;
  status: 'loaded';
  snapshot: GameSnapshot;
}

export interface RawDevtoolsStateSnapshot {
  visible: boolean;
  snapshot: RawGameSnapshot;
}

export interface DevtoolsStateSnapshot {
  visible: boolean;
  snapshot: GameSnapshot;
}

export interface RawDevtoolsGetStateResponse extends RawDevtoolsStateSnapshot {}

export interface DevtoolsGetStateResponse extends DevtoolsStateSnapshot {}

export interface DevtoolsSetVisibilityPayload {
  visible: boolean;
}

export interface RawDevtoolsSetVisibilityResponse extends RawDevtoolsStateSnapshot {}

export interface DevtoolsSetVisibilityResponse extends DevtoolsStateSnapshot {}

export interface DevtoolsApplyResourcesPayload {
  materials: number;
  data: number;
}

export type DevtoolsApplyResourcesRejectionCode = 'invalid_range' | 'invalid_state';

export type RawDevtoolsApplyResourcesResponse = GameActionResponse<
  DevtoolsApplyResourcesRejectionCode
>;

export type GatewayDevtoolsApplyResourcesResponse = GatewayActionResponse<
  DevtoolsApplyResourcesRejectionCode
>;

export interface DevtoolsApplyCrewPayload {
  crewTotal: number;
}

export type DevtoolsApplyCrewRejectionCode =
  | 'invalid_range'
  | 'constraint_violation'
  | 'invalid_state';

export type RawDevtoolsApplyCrewResponse = GameActionResponse<DevtoolsApplyCrewRejectionCode>;

export type GatewayDevtoolsApplyCrewResponse = GatewayActionResponse<
  DevtoolsApplyCrewRejectionCode
>;

export interface DevtoolsApplySystemsEntryPayload {
  id: SystemId;
  level: number;
}

export interface DevtoolsApplySystemsPayload {
  systems: DevtoolsApplySystemsEntryPayload[];
}

export type DevtoolsApplySystemsRejectionCode =
  | 'invalid_range'
  | 'unknown_id'
  | 'invalid_state';

export type RawDevtoolsApplySystemsResponse = GameActionResponse<
  DevtoolsApplySystemsRejectionCode
>;

export type GatewayDevtoolsApplySystemsResponse = GatewayActionResponse<
  DevtoolsApplySystemsRejectionCode
>;

export interface DevtoolsApplyServicesEntryPayload {
  id: ServiceId;
  desiredActive: boolean;
  assignedCrew: number;
  priority: number;
}

export interface DevtoolsApplyServicesPayload {
  services: DevtoolsApplyServicesEntryPayload[];
}

export type DevtoolsApplyServicesRejectionCode =
  | 'invalid_range'
  | 'unknown_id'
  | 'constraint_violation'
  | 'invalid_state';

export type RawDevtoolsApplyServicesResponse = GameActionResponse<
  DevtoolsApplyServicesRejectionCode
>;

export type GatewayDevtoolsApplyServicesResponse = GatewayActionResponse<
  DevtoolsApplyServicesRejectionCode
>;

export type DevtoolsProgressionSurveyProgressPayload = Partial<Record<PlanetId, number>>;

export interface DevtoolsApplyProgressionPayload {
  doctrineFragments: number;
  unlockedDoctrines: DoctrineId[];
  discoveredPlanets: PlanetId[];
  activePlanet: PlanetId;
  surveyProgress: DevtoolsProgressionSurveyProgressPayload;
}

export type DevtoolsApplyProgressionRejectionCode =
  | 'invalid_range'
  | 'unknown_id'
  | 'constraint_violation'
  | 'invalid_state';

export type RawDevtoolsApplyProgressionResponse = GameActionResponse<
  DevtoolsApplyProgressionRejectionCode
>;

export type GatewayDevtoolsApplyProgressionResponse = GatewayActionResponse<
  DevtoolsApplyProgressionRejectionCode
>;

export interface DevtoolsAdvanceTicksPayload {
  count: number;
}

export type DevtoolsAdvanceTicksRejectionCode = 'invalid_range' | 'invalid_state';

export type RawDevtoolsAdvanceTicksResponse = GameActionResponse<
  DevtoolsAdvanceTicksRejectionCode
>;

export type GatewayDevtoolsAdvanceTicksResponse = GatewayActionResponse<
  DevtoolsAdvanceTicksRejectionCode
>;

export interface DevtoolsResetToStarterPayload {}

export type DevtoolsResetToStarterRejectionCode = 'invalid_state';

export type RawDevtoolsResetToStarterResponse = GameActionResponse<
  DevtoolsResetToStarterRejectionCode
>;

export type GatewayDevtoolsResetToStarterResponse = GatewayActionResponse<
  DevtoolsResetToStarterRejectionCode
>;

export interface DevtoolsCommandPayloads {
  game_devtools_get_state: undefined;
  game_devtools_set_visibility: DevtoolsSetVisibilityPayload;
  game_devtools_apply_resources: DevtoolsApplyResourcesPayload;
  game_devtools_apply_crew: DevtoolsApplyCrewPayload;
  game_devtools_apply_systems: DevtoolsApplySystemsPayload;
  game_devtools_apply_services: DevtoolsApplyServicesPayload;
  game_devtools_apply_progression: DevtoolsApplyProgressionPayload;
  game_devtools_advance_ticks: DevtoolsAdvanceTicksPayload;
  game_devtools_reset_to_starter: DevtoolsResetToStarterPayload;
}

export interface DevtoolsCommandResponses {
  game_devtools_get_state: RawDevtoolsGetStateResponse;
  game_devtools_set_visibility: RawDevtoolsSetVisibilityResponse;
  game_devtools_apply_resources: RawDevtoolsApplyResourcesResponse;
  game_devtools_apply_crew: RawDevtoolsApplyCrewResponse;
  game_devtools_apply_systems: RawDevtoolsApplySystemsResponse;
  game_devtools_apply_services: RawDevtoolsApplyServicesResponse;
  game_devtools_apply_progression: RawDevtoolsApplyProgressionResponse;
  game_devtools_advance_ticks: RawDevtoolsAdvanceTicksResponse;
  game_devtools_reset_to_starter: RawDevtoolsResetToStarterResponse;
}

export type DevtoolsCommandName =
  | 'game_devtools_get_state'
  | 'game_devtools_set_visibility'
  | 'game_devtools_apply_resources'
  | 'game_devtools_apply_crew'
  | 'game_devtools_apply_systems'
  | 'game_devtools_apply_services'
  | 'game_devtools_apply_progression'
  | 'game_devtools_advance_ticks'
  | 'game_devtools_reset_to_starter';

export interface DevtoolsCommandTransport {
  invoke<TCommand extends DevtoolsCommandName>(
    command: TCommand,
    payload: DevtoolsCommandPayloads[TCommand],
  ): Promise<DevtoolsCommandResponses[TCommand]>;
}

export type GameGatewayTransport = GameTransport | (GameTransport & DevtoolsCommandTransport);

export interface GameCommandPayloads {
  game_get_snapshot: undefined;
  game_upgrade_system: UpgradeSystemInput;
  game_set_service_activation: SetServiceActivationInput;
  game_assign_service_crew: AssignServiceCrewInput;
  game_reprioritize_service: ReprioritizeServiceInput;
  game_start_survey: undefined;
  game_purchase_doctrine: PurchaseDoctrineInput;
  game_confirm_prestige: ConfirmPrestigeInput;
  game_request_save: undefined;
  game_request_load: undefined;
}

export interface GameCommandResponses {
  game_get_snapshot: RawGameSnapshot;
  game_upgrade_system: GameActionResponse<SystemUpgradeRejectionCode>;
  game_set_service_activation: GameActionResponse<ServiceActivationRejectionCode>;
  game_assign_service_crew: GameActionResponse<ServiceCrewAssignmentRejectionCode>;
  game_reprioritize_service: GameActionResponse<ServicePriorityRejectionCode>;
  game_start_survey: GameActionResponse<SurveyStartRejectionCode>;
  game_purchase_doctrine: GameActionResponse<DoctrinePurchaseRejectionCode>;
  game_confirm_prestige: GameActionResponse<PrestigeRejectionCode>;
  game_request_save: SaveGameResponse;
  game_request_load: LoadGameResponse;
}

export type GameCommandName =
  | 'game_get_snapshot'
  | 'game_upgrade_system'
  | 'game_set_service_activation'
  | 'game_assign_service_crew'
  | 'game_reprioritize_service'
  | 'game_start_survey'
  | 'game_purchase_doctrine'
  | 'game_confirm_prestige'
  | 'game_request_save'
  | 'game_request_load';

export interface GameTransport {
  invoke<TCommand extends GameCommandName>(
    command: TCommand,
    payload: GameCommandPayloads[TCommand],
  ): Promise<GameCommandResponses[TCommand]>;
}
