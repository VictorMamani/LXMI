import type {
  OfficialRelease,
  RuntimePackage,
  RuntimeSources,
  RuntimeStatus,
  SteamScan,
} from "../api/types";

export const VERIFIED = "official_release_verified";

export function versionOf(pkg: RuntimePackage | undefined): string | null {
  return pkg?.version?.raw ?? null;
}

/** Compares dotted versions ("2.4.1" < "2.10.0"); a leading "v" is ignored. */
export function compareVersions(a: string, b: string): number {
  const parts = (v: string) =>
    v
      .replace(/^v/i, "")
      .split(/[.-]/)
      .map((p) => Number.parseInt(p, 10) || 0);
  const [left, right] = [parts(a), parts(b)];
  for (let i = 0; i < Math.max(left.length, right.length); i++) {
    const diff = (left[i] ?? 0) - (right[i] ?? 0);
    if (diff !== 0) return Math.sign(diff);
  }
  return 0;
}

function newestVerified(
  packages: RuntimePackage[],
): RuntimePackage | undefined {
  return packages
    .filter((p) => p.authenticity === VERIFIED)
    .sort((a, b) =>
      compareVersions(versionOf(b) ?? "0", versionOf(a) ?? "0"),
    )[0];
}

/**
 * Packages that build the runtime: the ones of the applied profile when there
 * is one, otherwise the newest verified packages available.
 */
export function resolveSources(status: RuntimeStatus): RuntimeSources {
  const current = status.selection.current;
  if (current)
    return {
      importer: current.integration_package_id,
      libraries: current.libraries_package_id,
    };
  return {
    importer: newestVerified(status.importer_packages)?.id ?? "",
    libraries: newestVerified(status.libraries_packages)?.id ?? "",
  };
}

export function packagesInUse(status: RuntimeStatus, sources: RuntimeSources) {
  return {
    importer: status.importer_packages.find((p) => p.id === sources.importer),
    libraries: status.libraries_packages.find(
      (p) => p.id === sources.libraries,
    ),
  };
}

export type PendingUpdate = { kind: string; tag: string; version: string };

/** Official releases newer than the packages the runtime is built from. */
export function pendingUpdates(
  releases: OfficialRelease[],
  importerKind: string,
  installed: { importer: string | null; libraries: string | null },
): PendingUpdate[] {
  const wanted: [string, string | null][] = [
    [importerKind, installed.importer],
    ["xxmi_libraries", installed.libraries],
  ];
  return wanted.flatMap(([kind, current]) => {
    const release = releases.find((r) => r.package_kind === kind);
    if (!release) return [];
    const version = release.version || release.tag;
    return current === null || compareVersions(version, current) > 0
      ? [{ kind, tag: release.tag, version: version.replace(/^v/i, "") }]
      : [];
  });
}

/** Steam's plan for one game: install location and selected Proton. */
export function steamPlanFor(scan: SteamScan | null, backendId: string) {
  const plan = scan?.runtimePlans.find((p) => p.gameId === backendId);
  if (!plan) return null;
  return {
    detected: plan.installation.state === "detected",
    library:
      plan.installation.state === "detected"
        ? plan.installation.steamLibrary
        : null,
    proton:
      plan.selection.state === "selected"
        ? plan.selection.candidate.displayName
        : null,
  };
}
