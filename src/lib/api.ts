import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  CatalogSource,
  DiscoveryResponse,
  LocalRunResult,
  MigrationPlan,
  NameOverride,
  NamePreview,
  NamingRuleSpec,
  NozzleSelection,
  PrinterTarget,
  RestorePreview,
  RollbackOutcome,
  SourceApp,
  SourceKind,
  SyncProgressEvent,
  SyncResult,
} from "./types";

interface SourceCatalogResponse {
  catalog_id: string;
  sources: CatalogSource[];
}

interface TargetCatalogResponse {
  catalog_id: string;
  printers: PrinterTarget[];
}

interface BuildPlanRequest {
  source_catalog_id: string;
  source_ids: string[];
  target_catalog_id: string;
  nozzles: NozzleSelection[];
  destination_account_id: string;
  preset_template: string;
  ams_template: string;
  naming: {
    preset_rules: NamingRuleSpec[];
    ams_rules: NamingRuleSpec[];
    overrides: NameOverride[];
  };
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
  catalogSources: (rootIds: string[]) =>
    invoke<SourceCatalogResponse>("catalog_sources", {
      request: { root_ids: rootIds },
    }),
  catalogTargets: (catalogId: string, showCustom: boolean) =>
    invoke<TargetCatalogResponse>("catalog_targets", {
      request: { catalog_id: catalogId, show_custom: showCustom },
    }),
  previewNames: (request: {
    preset_template: string;
    ams_template: string;
    preset_rules: NamingRuleSpec[];
    ams_rules: NamingRuleSpec[];
    rows: PreviewNameRowInput[];
  }) => invoke<NamePreview[]>("preview_names", { request }),
  buildPlan: (request: BuildPlanRequest) =>
    invoke<{ plan: MigrationPlan }>("build_plan", { request }),
  executePlan: (planId: string) =>
    invoke<LocalRunResult>("execute_plan", { request: { plan_id: planId } }),
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
