import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { InvokeArgs } from '@tauri-apps/api/core';

import { adaptGameSnapshot } from './adapters';
import type {
  AssignServiceCrewInput,
  GameCommandName,
  GameCommandPayloads,
  GameCommandResponses,
  GameSnapshot,
  GatewayActionResponse,
  GatewayLoadGameResponse,
  GatewaySaveGameResponse,
  GameTransport,
  PurchaseDoctrineInput,
  RawGameSnapshot,
  ReprioritizeServiceInput,
  SelectPlanetInput,
  SetServiceActivationInput,
  SystemUpgradeRejectionCode,
  UpgradeSystemInput,
  ServiceActivationRejectionCode,
  ServiceCrewAssignmentRejectionCode,
  ServicePriorityRejectionCode,
  PlanetSelectionRejectionCode,
  SurveyStartRejectionCode,
  DoctrinePurchaseRejectionCode,
  PrestigeRejectionCode,
  ConfirmPrestigeInput,
} from './types';
import { maybeCreatePreviewFixtureTransport } from './testing/transport';

const tauriTransport: GameTransport = {
  invoke(command, payload) {
    return tauriInvoke(command, payload as InvokeArgs | undefined) as Promise<
      GameCommandResponses[typeof command]
    >;
  },
};

export function createGameGateway(transport: GameTransport = resolveDefaultTransport()) {
  return {
    transport,
    getSnapshot: () => invokeSnapshot('game_get_snapshot', undefined, transport),
    upgradeSystem: (input: UpgradeSystemInput) =>
      invokeAction<'game_upgrade_system', SystemUpgradeRejectionCode>(
        'game_upgrade_system',
        input,
        transport,
      ),
    setServiceActivation: (input: SetServiceActivationInput) =>
      invokeAction<'game_set_service_activation', ServiceActivationRejectionCode>(
        'game_set_service_activation',
        input,
        transport,
      ),
    assignServiceCrew: (input: AssignServiceCrewInput) =>
      invokeAction<'game_assign_service_crew', ServiceCrewAssignmentRejectionCode>(
        'game_assign_service_crew',
        input,
        transport,
      ),
    reprioritizeService: (input: ReprioritizeServiceInput) =>
      invokeAction<'game_reprioritize_service', ServicePriorityRejectionCode>(
        'game_reprioritize_service',
        input,
        transport,
      ),
    selectPlanet: (input: SelectPlanetInput) =>
      invokeAction<'game_select_planet', PlanetSelectionRejectionCode>(
        'game_select_planet',
        input,
        transport,
      ),
    startSurvey: () =>
      invokeAction<'game_start_survey', SurveyStartRejectionCode>(
        'game_start_survey',
        undefined,
        transport,
      ),
    purchaseDoctrine: (input: PurchaseDoctrineInput) =>
      invokeAction<'game_purchase_doctrine', DoctrinePurchaseRejectionCode>(
        'game_purchase_doctrine',
        input,
        transport,
      ),
    confirmPrestige: (input: ConfirmPrestigeInput) =>
      invokeAction<'game_confirm_prestige', PrestigeRejectionCode>(
        'game_confirm_prestige',
        input,
        transport,
      ),
    requestSave: () => invokeSaveLike('game_request_save', undefined, transport),
    requestLoad: () => invokeLoadLike('game_request_load', undefined, transport),
  };
}

export const gameGateway = createGameGateway();

export function resolveDefaultTransport(): GameTransport {
  return maybeCreatePreviewFixtureTransport() ?? tauriTransport;
}

async function invokeSnapshot<TCommand extends 'game_get_snapshot'>(
  command: TCommand,
  payload: GameCommandPayloads[TCommand],
  transport: GameTransport,
): Promise<GameSnapshot> {
  const response = await transport.invoke(command, payload);
  return adaptGameSnapshot(response as RawGameSnapshot);
}

async function invokeAction<
  TCommand extends Exclude<
    GameCommandName,
    'game_get_snapshot' | 'game_request_save' | 'game_request_load'
  >,
  TReason extends string,
>(
  command: TCommand,
  payload: GameCommandPayloads[TCommand],
  transport: GameTransport,
): Promise<GatewayActionResponse<TReason>> {
  const response = (await transport.invoke(command, payload)) as GameCommandResponses[TCommand];

  if (response.ok) {
    return {
      ok: true,
      snapshot: adaptGameSnapshot(response.snapshot),
    };
  }

  return {
    ok: false,
    reasonCode: response.reasonCode as TReason,
    snapshot: adaptGameSnapshot(response.snapshot),
  };
}

async function invokeSaveLike<TCommand extends 'game_request_save'>(
  command: TCommand,
  payload: GameCommandPayloads[TCommand],
  transport: GameTransport,
): Promise<GatewaySaveGameResponse> {
  const response = await transport.invoke(command, payload);
  return {
    ok: true,
    status: response.status,
    snapshot: adaptGameSnapshot(response.snapshot),
  };
}

async function invokeLoadLike<TCommand extends 'game_request_load'>(
  command: TCommand,
  payload: GameCommandPayloads[TCommand],
  transport: GameTransport,
): Promise<GatewayLoadGameResponse> {
  const response = await transport.invoke(command, payload);
  return {
    ok: true,
    status: response.status,
    snapshot: adaptGameSnapshot(response.snapshot),
  };
}
