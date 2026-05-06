import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { InvokeArgs } from '@tauri-apps/api/core';

import { adaptGameSnapshot } from './adapters';
import type {
  AssignServiceCrewInput,
  DevtoolsCommandName,
  DevtoolsCommandPayloads,
  DevtoolsCommandResponses,
  DevtoolsCommandTransport,
  DevtoolsAdvanceTicksPayload,
  DevtoolsAdvanceTicksRejectionCode,
  DevtoolsApplyCrewPayload,
  DevtoolsApplyCrewRejectionCode,
  DevtoolsApplyProgressionPayload,
  DevtoolsApplyProgressionRejectionCode,
  DevtoolsApplyResourcesPayload,
  DevtoolsApplyResourcesRejectionCode,
  DevtoolsApplyServicesPayload,
  DevtoolsApplyServicesRejectionCode,
  DevtoolsApplySystemsPayload,
  DevtoolsApplySystemsRejectionCode,
  DevtoolsGetStateResponse,
  DevtoolsResetToStarterPayload,
  DevtoolsResetToStarterRejectionCode,
  DevtoolsSetVisibilityPayload,
  DevtoolsSetVisibilityResponse,
  GameCommandName,
  GameCommandPayloads,
  GameCommandResponses,
  GameSnapshot,
  GatewayActionResponse,
  GatewayLoadGameResponse,
  GatewaySaveGameResponse,
  GameGatewayTransport,
  GameTransport,
  PurchaseDoctrineInput,
  RawGameSnapshot,
  ReprioritizeServiceInput,
  SetServiceActivationInput,
  SystemUpgradeRejectionCode,
  UpgradeSystemInput,
  ServiceActivationRejectionCode,
  ServiceCrewAssignmentRejectionCode,
  ServicePriorityRejectionCode,
  SurveyStartRejectionCode,
  DoctrinePurchaseRejectionCode,
  PrestigeRejectionCode,
  RawDevtoolsStateSnapshot,
  ConfirmPrestigeInput,
} from './types';
import { maybeCreatePreviewFixtureTransport } from './testing/transport';

const NO_INPUT_COMMANDS = new Set([
  'game_get_snapshot',
  'game_start_survey',
  'game_request_save',
  'game_request_load',
  'game_devtools_get_state',
  'game_devtools_reset_to_starter',
]);

const tauriTransport: GameGatewayTransport = {
  invoke(command, payload) {
    const rustCommand =
      command === 'game_set_service_activation'
        ? 'game_toggle_service'
        : command === 'game_confirm_prestige'
          ? 'game_execute_prestige'
          : command;

    const args = NO_INPUT_COMMANDS.has(rustCommand) || payload === undefined ? undefined : { input: payload };

    return tauriInvoke(rustCommand, args as InvokeArgs | undefined) as Promise<
      GameCommandResponses[typeof command]
    >;
  },
};

export function createGameGateway(transport: GameGatewayTransport = resolveDefaultTransport()) {
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
    getDevtoolsState: () => invokeDevtoolsState('game_devtools_get_state', undefined, transport),
    setDevtoolsVisibility: (input: DevtoolsSetVisibilityPayload) =>
      invokeDevtoolsState('game_devtools_set_visibility', input, transport),
    applyResources: (input: DevtoolsApplyResourcesPayload) =>
      invokeAction<'game_devtools_apply_resources', DevtoolsApplyResourcesRejectionCode>(
        'game_devtools_apply_resources',
        input,
        transport,
      ),
    applyCrew: (input: DevtoolsApplyCrewPayload) =>
      invokeAction<'game_devtools_apply_crew', DevtoolsApplyCrewRejectionCode>(
        'game_devtools_apply_crew',
        input,
        transport,
      ),
    applySystems: (input: DevtoolsApplySystemsPayload) =>
      invokeAction<'game_devtools_apply_systems', DevtoolsApplySystemsRejectionCode>(
        'game_devtools_apply_systems',
        input,
        transport,
      ),
    applyServices: (input: DevtoolsApplyServicesPayload) =>
      invokeAction<'game_devtools_apply_services', DevtoolsApplyServicesRejectionCode>(
        'game_devtools_apply_services',
        input,
        transport,
      ),
    applyProgression: (input: DevtoolsApplyProgressionPayload) =>
      invokeAction<'game_devtools_apply_progression', DevtoolsApplyProgressionRejectionCode>(
        'game_devtools_apply_progression',
        input,
        transport,
      ),
    advanceTicks: (input: DevtoolsAdvanceTicksPayload) =>
      invokeAction<'game_devtools_advance_ticks', DevtoolsAdvanceTicksRejectionCode>(
        'game_devtools_advance_ticks',
        input,
        transport,
      ),
    resetToStarter: (input: DevtoolsResetToStarterPayload = {}) =>
      invokeAction<'game_devtools_reset_to_starter', DevtoolsResetToStarterRejectionCode>(
        'game_devtools_reset_to_starter',
        input,
        transport,
      ),
  };
}

export const gameGateway = createGameGateway();

export function resolveDefaultTransport(): GameGatewayTransport {
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

async function invokeDevtoolsState<
  TCommand extends 'game_devtools_get_state' | 'game_devtools_set_visibility',
>(
  command: TCommand,
  payload: DevtoolsCommandPayloads[TCommand],
  transport: GameGatewayTransport,
): Promise<TCommand extends 'game_devtools_get_state' ? DevtoolsGetStateResponse : DevtoolsSetVisibilityResponse> {
  const response = (await (transport as DevtoolsCommandTransport).invoke(
    command,
    payload,
  )) as RawDevtoolsStateSnapshot;

  return {
    visible: response.visible,
    snapshot: adaptGameSnapshot(response.snapshot),
  } as TCommand extends 'game_devtools_get_state'
    ? DevtoolsGetStateResponse
    : DevtoolsSetVisibilityResponse;
}

async function invokeAction<
  TCommand extends Exclude<
    GameCommandName | DevtoolsCommandName,
    | 'game_get_snapshot'
    | 'game_request_save'
    | 'game_request_load'
    | 'game_devtools_get_state'
    | 'game_devtools_set_visibility'
  >,
  TReason extends string,
>(
  command: TCommand,
  payload: (GameCommandPayloads & DevtoolsCommandPayloads)[TCommand],
  transport: GameGatewayTransport,
): Promise<GatewayActionResponse<TReason>> {
  const response = (command.startsWith('game_devtools_')
    ? await (transport as DevtoolsCommandTransport).invoke(
        command as DevtoolsCommandName,
        payload as DevtoolsCommandPayloads[DevtoolsCommandName],
      )
    : await transport.invoke(
        command as GameCommandName,
        payload as GameCommandPayloads[GameCommandName],
      )) as
    | GameCommandResponses[GameCommandName]
    | DevtoolsCommandResponses[DevtoolsCommandName];

  if (!('ok' in response)) {
    throw new Error(`Command ${command} did not return an action response`);
  }

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
