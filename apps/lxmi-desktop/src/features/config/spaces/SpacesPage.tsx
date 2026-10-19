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
    toast(`Runtime ${game.importer} actualizado`, "success");
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
    toast("Selección anterior recuperada", "success");
  }

  const versions = [
    summary.importerVersion && `${game.importer} ${summary.importerVersion}`,
    summary.librariesVersion && `XXMI Libraries ${summary.librariesVersion}`,
  ].filter(Boolean);
  const target =
    updates.find((u) => u.kind === game.importerKind) ?? updates[0];

  return (
    <PageLayout eyebrow={`Espacio ${game.channel}`} title={game.name}>
      {(action.error || runtime.error) && (
        <InlineError>{action.error ?? runtime.error}</InlineError>
      )}

      <div>
        <SettingRow
          label="Juego"
          value={
            steam.loading && !steam.data
              ? "Buscando en Steam…"
              : summary.steam?.detected
                ? "Detectado en Steam"
                : "No encontrado en Steam"
          }
          status={
            steam.data &&
            (summary.steam?.detected ? (
              <StatusDot status="ok" size="sm">
                OK
              </StatusDot>
            ) : (
              <StatusDot status="warn" size="sm">
                Revisar
              </StatusDot>
            ))
          }
          detail={
            summary.steam?.library
              ? `Biblioteca ${summary.steam.library}${summary.steam.proton ? ` · ${summary.steam.proton}` : ""}`
              : (steam.error ??
                "Instala el juego desde Steam para jugar con mods.")
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
              {steam.loading ? "Buscando…" : "Volver a buscar"}
            </Button>
          }
        />

        <SettingRow
          label="Runtime de mods"
          value={
            hasRuntime && versions.length
              ? versions.join(" · ")
              : "Runtime sin preparar"
          }
          status={
            target && (
              <StatusDot status="accent" size="sm">
                Actualización
              </StatusDot>
            )
          }
          detail={
            hasRuntime
              ? verified
                ? "Paquetes oficiales verificados por firma y hash"
                : "Hay paquetes sin verificar por firma oficial"
              : `Descarga ${game.importer} y XXMI Libraries oficiales y ensámblalos en una copia privada`
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
                  ? "Actualizando…"
                  : hasRuntime && target
                    ? `Actualizar a ${target.version}`
                    : "Preparar runtime"}
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
                  ? "Consultando…"
                  : "Buscar actualización"}
              </Button>
            )
          }
        />

        <SettingRow
          label="Próximo inicio"
          value={
            summary.profileName
              ? `Perfil «${summary.profileName}» · ${plural(summary.modCount, "mod")}`
              : "Sin perfil aplicado"
          }
          detail={
            previous
              ? `Anterior: «${previous.profile_name}» · reversible`
              : summary.profileName
                ? "Aplicado a una copia privada · reversible"
                : "Elige y aplica un perfil en Mods"
          }
          action={
            <Button
              icon={RotateCcw}
              disabled={action.busy || !previous}
              onClick={() => void undo()}
            >
              {action.pending === "undo" ? "Deshaciendo…" : "Deshacer"}
            </Button>
          }
        />

        <SettingRow
          label="Carpeta de mods"
          value={
            storage.data?.storage_root ??
            (storage.loading ? "Consultando…" : "No disponible")
          }
          detail="Biblioteca privada · el juego nunca se modifica directamente"
          action={
            <Button
              icon={Copy}
              disabled={!storage.data}
              onClick={() => {
                if (!storage.data) return;
                void navigator.clipboard
                  .writeText(storage.data.storage_root)
                  .then(() => toast("Ruta copiada", "success"))
                  .catch(() => toast("No se pudo copiar la ruta", "error"));
              }}
            >
              Copiar ruta
            </Button>
          }
        />
      </div>

      <SpaceShelf />
    </PageLayout>
  );
}
