import { invoke } from "@tauri-apps/api/core";
import type {
  CatalogSource,
  DiscoveryResponse,
  LocalRunResult,
  MigrationPlan,
  NozzleSelection,
  PrinterTarget,
  RestorePreview,
  RollbackOutcome,
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
    rows: Array<Record<string, string>>;
  }) =>
    invoke<Array<{ preset_name: string; ams_name: string }>>("preview_names", {
      request,
    }),
  buildPlan: (request: BuildPlanRequest) =>
    invoke<{ plan: MigrationPlan }>("build_plan", { request }),
  executePlan: (planId: string) =>
    invoke<LocalRunResult>("execute_plan", { request: { plan_id: planId } }),
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
