import { Copy, Download, RefreshCw, RotateCcw, Search } from "lucide-react";
import { call } from "../../../api/client";
import { Button } from "../../../components/Button";
import { InlineError } from "../../../components/InlineError";
import { StatusDot } from "../../../components/StatusDot";
import { plural } from "../../../domain/format";
import { pendingUpdates, VERIFIED } from "../../../domain/runtime";
import { useAction } from "../../../hooks/useAction";
import { useAsync } from "../../../hooks/useAsync";
import {
  officialReleases,
  updateRuntime,
} from "../../../services/runtimeUpdate";
import { useGameData } from "../../../state/GameDataContext";
import { useSystem } from "../../../state/SystemContext";
import { useToast } from "../../../state/ToastContext";
import { useRuntimeSummary } from "../../../state/useRuntimeSummary";
import { PageLayout } from "../PageLayout";
import { SettingRow } from "../SettingRow";
import { SpaceShelf } from "./SpaceShelf";

type Pending = "rescan" | "update" | "releases" | "undo";

const loadStorage = () => call("xxmi_status");

export function SpacesPage() {
  const { game, runtime, refresh } = useGameData();
  const { steam } = useSystem();
  const summary = useRuntimeSummary();
  const toast = useToast();
  const action = useAction<Pending>();
  const releases = useAsync(() => officialReleases(), "releases");
  const storage = useAsync(loadStorage, "storage");

  const status = summary.status;
  const updates = releases.data
    ? pendingUpdates(releases.data, game.importerKind, {
        importer: summary.importerVersion,
        libraries: summary.librariesVersion,
      })
    : [];
  const hasRuntime = Boolean(status?.base_runtime);
  const verified =
    status !== null &&
    [...status.importer_packages, ...status.libraries_packages]
      .filter(
        (p) =>
          p.id === summary.sources?.importer ||
          p.id === summary.sources?.libraries,
      )
      .every((p) => p.authenticity === VERIFIED);
  const previous = status?.selection.previous ?? null;

  async function update() {
    if (!runtime.data) return;
    const done = await action.run("update", async () => {
      const pending = hasRuntime
        ? updates
        : pendingUpdates(await officialReleases(), game.importerKind, {
            importer: summary.importerVersion,
            libraries: summary.librariesVersion,
          });
      await updateRuntime({
        gameId: game.backendId,
        ...runtime.data!,
        updates: pending,
      });
      return true;
    });
    if (!done) return;
    await refresh().catch(() => {});
    toast(`${game.importer} runtime updated`, "success");
  }

  async function undo() {
    if (!status) return;
    const done = await action.run("undo", () =>
      call("rollback_mod_profile_activation", {
        gameId: game.backendId,
        selectionRevision: status.selection.revision,
      }),
    );
    if (done === undefined) return;
    await refresh().catch(() => {});
    toast("Previous selection restored", "success");
  }

  const versions = [
    summary.importerVersion && `${game.importer} ${summary.importerVersion}`,
    summary.librariesVersion && `XXMI Libraries ${summary.librariesVersion}`,
  ].filter(Boolean);
  const target =
    updates.find((u) => u.kind === game.importerKind) ?? updates[0];

  return (
    <PageLayout eyebrow={`Space ${game.channel}`} title={game.name}>
      {(action.error || runtime.error) && (
        <InlineError>{action.error ?? runtime.error}</InlineError>
      )}

      <div>
        <SettingRow
          label="Game"
          value={
            steam.loading && !steam.data
              ? "Scanning Steam…"
              : summary.steam?.detected
                ? "Detected in Steam"
                : "Not found in Steam"
          }
          status={
            steam.data &&
            (summary.steam?.detected ? (
              <StatusDot status="ok" size="sm">
                OK
              </StatusDot>
            ) : (
              <StatusDot status="warn" size="sm">
                Review
              </StatusDot>
            ))
          }
          detail={
            summary.steam?.library
              ? `Library ${summary.steam.library}${summary.steam.proton ? ` · ${summary.steam.proton}` : ""}`
              : (steam.error ??
                "Install the game through Steam to play with mods.")
          }
          action={
            <Button
              icon={RefreshCw}
              disabled={steam.loading}
              onClick={() =>
                void steam
                  .reload()
                  .then(() => runtime.reload())
                  .catch(() => {})
              }
            >
              {steam.loading ? "Scanning…" : "Scan again"}
            </Button>
          }
        />

        <SettingRow
          label="Mod runtime"
          value={
            hasRuntime && versions.length
              ? versions.join(" · ")
              : "Runtime not prepared"
          }
          status={
            target && (
              <StatusDot status="accent" size="sm">
                Update
              </StatusDot>
            )
          }
          detail={
            hasRuntime
              ? verified
                ? "Official packages verified by signature and hash"
                : "Some packages lack official signature verification"
              : `Download official ${game.importer} and XXMI Libraries packages and assemble a private copy`
          }
          action={
            !hasRuntime || target ? (
              <Button
                variant="primary"
                icon={Download}
                disabled={action.busy || !status?.installed}
                onClick={() => void update()}
              >
                {action.pending === "update"
                  ? "Updating…"
                  : hasRuntime && target
                    ? `Update to ${target.version}`
                    : "Prepare runtime"}
              </Button>
            ) : (
              <Button
                icon={Search}
                disabled={action.busy || releases.loading}
                onClick={() =>
                  void action.run("releases", () =>
                    officialReleases(true).then(() => releases.reload()),
                  )
                }
              >
                {releases.loading || action.pending === "releases"
                  ? "Checking…"
                  : "Check for updates"}
              </Button>
            )
          }
        />

        <SettingRow
          label="Next launch"
          value={
            summary.profileName
              ? `Profile '${summary.profileName}' · ${plural(summary.modCount, "mod")}`
              : "No profile applied"
          }
          detail={
            previous
              ? `Previous: '${previous.profile_name}' · reversible`
              : summary.profileName
                ? "Applied to a private copy · reversible"
                : "Choose and apply a profile in Mods"
          }
          action={
            <Button
              icon={RotateCcw}
              disabled={action.busy || !previous}
              onClick={() => void undo()}
            >
              {action.pending === "undo" ? "Undoing…" : "Undo"}
            </Button>
          }
        />

        <SettingRow
          label="Mods directory"
          value={
            storage.data?.storage_root ??
            (storage.loading ? "Checking…" : "Unavailable")
          }
          detail="Private library · the game is never modified directly"
          action={
            <Button
              icon={Copy}
              disabled={!storage.data}
              onClick={() => {
                if (!storage.data) return;
                void navigator.clipboard
                  .writeText(storage.data.storage_root)
                  .then(() => toast("Path copied", "success"))
                  .catch(() => toast("Could not copy the path", "error"));
              }}
            >
              Copy path
            </Button>
          }
        />
      </div>

      <SpaceShelf />
    </PageLayout>
  );
}
