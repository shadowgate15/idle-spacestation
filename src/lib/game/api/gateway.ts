import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { InvokeArgs } from '@tauri-apps/api/core';

import { adaptGameSnapshot } from './adapters';
import type {
  DevtoolsCommandName,
  DevtoolsCommandPayloads,
  DevtoolsCommandResponses,
  DevtoolsCommandTransport,
  DevtoolsAdvanceTicksRejectionCode,
  DevtoolsApplyCrewRejectionCode,
  DevtoolsApplyProgressionRejectionCode,
  DevtoolsApplyResourcesRejectionCode,
  DevtoolsApplyServicesRejectionCode,
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
  RawGameSnapshot,
  SystemUpgradeRejectionCode,
  ServiceActivationRejectionCode,
  ServiceCrewAssignmentRejectionCode,
  ServicePriorityRejectionCode,
  SurveyStartRejectionCode,
  DoctrinePurchaseRejectionCode,
  PrestigeRejectionCode,
  RawDevtoolsStateSnapshot,
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

const COMMAND_ALIASES: Partial<Record<GameCommandName | DevtoolsCommandName, string>> = {
  game_set_service_activation: 'game_toggle_service',
  game_confirm_prestige: 'game_execute_prestige',
};

const tauriTransport: GameGatewayTransport = {
  invoke(command, payload) {
    const rustCommand = COMMAND_ALIASES[command] ?? command;

    const args =
      NO_INPUT_COMMANDS.has(rustCommand) || payload === undefined ? undefined : { input: payload };

    return tauriInvoke(rustCommand, args as InvokeArgs | undefined) as Promise<
      GameCommandResponses[typeof command]
    >;
  },
  subscribeToStateChanges(
    callback: (raw: RawGameSnapshot) => void,
    onError?: (err: Error) => void,
  ): () => void {
    let unlistenFn: (() => void) | null = null;
    let cancelled = false;
    import('@tauri-apps/api/event')
      .then(({ listen }) =>
        listen<RawGameSnapshot>('game://state-changed', (event) => {
          if (!cancelled) callback(event.payload);
        }),
      )
      .then((fn) => {
        if (cancelled) fn();
        else unlistenFn = fn;
      })
      .catch((err) => {
        const error = err instanceof Error ? err : new Error(String(err));
        if (onError && !cancelled) {
          onError(error);
        } else {
          console.error('[tauriTransport] subscribeToStateChanges failed:', error);
        }
      });
    return () => {
      cancelled = true;
      unlistenFn?.();
    };
  },
};

type ActionCommandName = Exclude<
  GameCommandName | DevtoolsCommandName,
  | 'game_get_snapshot'
  | 'game_request_save'
  | 'game_request_load'
  | 'game_devtools_get_state'
  | 'game_devtools_set_visibility'
>;

type ActionPayload<C extends ActionCommandName> = (GameCommandPayloads &
  DevtoolsCommandPayloads)[C];

export function createGameGateway(transport: GameGatewayTransport = resolveDefaultTransport()) {
  function action<C extends ActionCommandName, R extends string>(cmd: C) {
    return (input: ActionPayload<C>) => invokeAction<C, R>(cmd, input, transport);
  }

  const startSurvey = action<'game_start_survey', SurveyStartRejectionCode>('game_start_survey');
  const resetToStarter = action<
    'game_devtools_reset_to_starter',
    DevtoolsResetToStarterRejectionCode
  >('game_devtools_reset_to_starter');

  return {
    transport,
    subscribeToStateChanges: (cb: (raw: RawGameSnapshot) => void, onError?: (err: Error) => void) =>
      transport.subscribeToStateChanges(cb, onError),
    getSnapshot: () => invokeSnapshot('game_get_snapshot', undefined, transport),
    requestSave: () => invokeSaveLike('game_request_save', undefined, transport),
    requestLoad: () => invokeLoadLike('game_request_load', undefined, transport),
    getDevtoolsState: () => invokeDevtoolsState('game_devtools_get_state', undefined, transport),
    setDevtoolsVisibility: (input: DevtoolsSetVisibilityPayload) =>
      invokeDevtoolsState('game_devtools_set_visibility', input, transport),
    upgradeSystem: action<'game_upgrade_system', SystemUpgradeRejectionCode>('game_upgrade_system'),
    setServiceActivation: action<'game_set_service_activation', ServiceActivationRejectionCode>(
      'game_set_service_activation',
    ),
    assignServiceCrew: action<'game_assign_service_crew', ServiceCrewAssignmentRejectionCode>(
      'game_assign_service_crew',
    ),
    reprioritizeService: action<'game_reprioritize_service', ServicePriorityRejectionCode>(
      'game_reprioritize_service',
    ),
    startSurvey: () => startSurvey(undefined),
    purchaseDoctrine: action<'game_purchase_doctrine', DoctrinePurchaseRejectionCode>(
      'game_purchase_doctrine',
    ),
    confirmPrestige: action<'game_confirm_prestige', PrestigeRejectionCode>(
      'game_confirm_prestige',
    ),
    applyResources: action<'game_devtools_apply_resources', DevtoolsApplyResourcesRejectionCode>(
      'game_devtools_apply_resources',
    ),
    applyCrew: action<'game_devtools_apply_crew', DevtoolsApplyCrewRejectionCode>(
      'game_devtools_apply_crew',
    ),
    applySystems: action<'game_devtools_apply_systems', DevtoolsApplySystemsRejectionCode>(
      'game_devtools_apply_systems',
    ),
    applyServices: action<'game_devtools_apply_services', DevtoolsApplyServicesRejectionCode>(
      'game_devtools_apply_services',
    ),
    applyProgression: action<
      'game_devtools_apply_progression',
      DevtoolsApplyProgressionRejectionCode
    >('game_devtools_apply_progression'),
    advanceTicks: action<'game_devtools_advance_ticks', DevtoolsAdvanceTicksRejectionCode>(
      'game_devtools_advance_ticks',
    ),
    resetToStarter: (input: DevtoolsResetToStarterPayload = {}) => resetToStarter(input),
  };
}

export const gameGateway = createGameGateway();

export type GameGateway = ReturnType<typeof createGameGateway>;

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
): Promise<
  TCommand extends 'game_devtools_get_state'
    ? DevtoolsGetStateResponse
    : DevtoolsSetVisibilityResponse
> {
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

async function invokeAction<TCommand extends ActionCommandName, TReason extends string>(
  command: TCommand,
  payload: ActionPayload<TCommand>,
  transport: GameGatewayTransport,
): Promise<GatewayActionResponse<TReason>> {
  const response = (
    command.startsWith('game_devtools_')
      ? await (transport as DevtoolsCommandTransport).invoke(
          command as DevtoolsCommandName,
          payload as DevtoolsCommandPayloads[DevtoolsCommandName],
        )
      : await transport.invoke(
          command as GameCommandName,
          payload as GameCommandPayloads[GameCommandName],
        )
  ) as GameCommandResponses[GameCommandName] | DevtoolsCommandResponses[DevtoolsCommandName];

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
