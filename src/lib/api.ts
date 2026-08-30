import { Channel, invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type {
  BuildPlanRequest,
  BuildPlanResponse,
  CatalogProgressEvent,
  CatalogSource,
  DiscoveryResponse,
  ExecutionProgressEvent,
  LocalRunResult,
  NamePreview,
  NamingRuleSpec,
  PrinterTarget,
  RestorePreview,
  RollbackOutcome,
  SourceApp,
  SourceKind,
  SyncProgressEvent,
  SyncResult,
  TargetTemplateDecision,
} from "./types";

interface SourceCatalogResponse {
  catalog_id: string;
  sources: CatalogSource[];
}

interface TargetCatalogResponse {
  catalog_id: string;
  printers: PrinterTarget[];
}

interface PreviewNameRowInput {
  source_name: string;
  vendor: string;
  material: string;
  family: string;
  variant: string;
  source_app: SourceApp;
  source_kind: SourceKind;
  printer: string;
  printer_code: string;
  nozzle: string;
}

export const api = {
  discover: () => invoke<DiscoveryResponse>("discover"),
  chooseManualSourceFolder: (request: {
    source_app: SourceApp;
    source_kind: SourceKind;
  }) =>
    invoke<DiscoveryResponse | null>("choose_manual_source_folder", {
      request,
    }),
  removeManualSourceFolder: (id: string) =>
    invoke<DiscoveryResponse>("remove_manual_source_folder", {
      request: { id },
    }),
  catalogSources: (
    rootIds: string[],
    onProgress: (event: CatalogProgressEvent) => void,
  ) => {
    const channel = new Channel<CatalogProgressEvent>();
    channel.onmessage = onProgress;
    return invoke<SourceCatalogResponse>("catalog_sources", {
      request: { root_ids: rootIds },
      onProgress: channel,
    });
  },
  catalogTargets: (
    catalogId: string,
    onProgress: (event: CatalogProgressEvent) => void,
  ) => {
    const channel = new Channel<CatalogProgressEvent>();
    channel.onmessage = onProgress;
    return invoke<TargetCatalogResponse>("catalog_targets", {
      request: { catalog_id: catalogId, show_custom: false },
      onProgress: channel,
    });
  },
  previewNames: (request: {
    preset_template: string;
    ams_template: string;
    preset_rules: NamingRuleSpec[];
    ams_rules: NamingRuleSpec[];
    rows: PreviewNameRowInput[];
  }) => invoke<NamePreview[]>("preview_names", { request }),
  buildPlan: (request: BuildPlanRequest) =>
    invoke<BuildPlanResponse>("build_plan", { request }),
  resolvePlanDependencies: (
    request: BuildPlanRequest,
    decisions: TargetTemplateDecision[],
  ) =>
    invoke<BuildPlanResponse>("resolve_plan_dependencies", {
      request: { request, decisions },
    }),
  printerArtwork: (catalogId: string, printerId: string) =>
    invoke<ArrayBuffer>("printer_artwork", { catalogId, printerId }),
  openSupportPage: () => openUrl("https://buymeacoffee.com/Remitec"),
  executePlan: (
    planId: string,
    onProgress: (event: ExecutionProgressEvent) => void,
  ) => {
    const channel = new Channel<ExecutionProgressEvent>();
    channel.onmessage = onProgress;
    return invoke<LocalRunResult>("execute_plan", {
      request: { plan_id: planId },
      onProgress: channel,
    });
  },
  synchronizeRun: (
    runId: string,
    onProgress: (event: SyncProgressEvent) => void,
  ) => {
    const channel = new Channel<SyncProgressEvent>();
    channel.onmessage = onProgress;
    return invoke<SyncResult>("synchronize_run", {
      request: { run_id: runId },
      onProgress: channel,
    });
  },
  cancelRun: (runId: string) => invoke<void>("cancel_run", { runId }),
  recordAmsVerification: (
    runId: string,
    operationIds: string[],
    note: string,
  ) =>
    invoke<void>("record_ams_verification", {
      request: { run_id: runId, operation_ids: operationIds, note },
    }),
  restorePreview: (runId: string) =>
    invoke<RestorePreview>("restore_preview", { runId }),
  restoreOwned: (runId: string) =>
    invoke<RollbackOutcome>("restore_owned", { runId }),
};
