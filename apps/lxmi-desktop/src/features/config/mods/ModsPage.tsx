import { FolderPlus, Search } from "lucide-react";
import { useState } from "react";
import { Button } from "../../../components/Button";
import { InlineError } from "../../../components/InlineError";
import { TextField } from "../../../components/TextField";
import { plural } from "../../../domain/format";
import { useAction } from "../../../hooks/useAction";
import { saveAndApplyProfile } from "../../../services/profileActivation";
import { useGameData } from "../../../state/GameDataContext";
import { useToast } from "../../../state/ToastContext";
import { PageLayout } from "../PageLayout";
import { ImportModDialog } from "./ImportModDialog";
import { ModTable } from "./ModTable";
import styles from "./ModsPage.module.css";
import { NewProfileDialog } from "./NewProfileDialog";
import { PendingChangesBar } from "./PendingChangesBar";
import { ProfilePicker } from "./ProfilePicker";
import { useProfileDraft } from "./useProfileDraft";

export function ModsPage() {
  const { game, library, runtime, refresh } = useGameData();
  const toast = useToast();
  const action = useAction<"apply">();
  const [query, setQuery] = useState("");
  const [dialog, setDialog] = useState<"import" | "profile" | null>(null);

  const lib = library.data;
  const status = runtime.data?.status ?? null;
  const applied = status?.selection.current?.profile_name ?? null;
  const draft = useProfileDraft(lib, applied);

  const mods = lib?.mods ?? [];
  const filtered = query
    ? mods.filter((mod) =>
        mod.name.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
      )
    : mods;
  const changes = draft.diff.count + (draft.isSaved ? 0 : 1);
  const needsApply =
    draft.name !== null && (changes > 0 || draft.name !== applied);

  async function apply() {
    if (!lib || !runtime.data || !draft.name) return;
    const result = await action.run("apply", () =>
      saveAndApplyProfile({
        gameId: game.backendId,
        library: lib,
        profileName: draft.name!,
        modIds: draft.selected,
        changed: changes > 0,
        status: runtime.data!.status,
        sources: runtime.data!.sources,
      }),
    );
    if (!result) return;
    await refresh().catch(() => {});
    draft.commit();
    if (result.applied) toast(`Perfil «${draft.name}» aplicado`, "success");
    else toast(`Perfil guardado. ${result.reason}`, "error");
  }

  let barTitle = "Sin cambios pendientes";
  if (draft.diff.count > 0)
    barTitle = `${plural(draft.diff.count, "cambio")} sin aplicar`;
  else if (!draft.isSaved && draft.name) barTitle = "Perfil nuevo sin guardar";
  else if (needsApply) barTitle = "Perfil sin aplicar";

  const active = `${draft.selected.length} de ${plural(mods.length, "mod")} activos`;
  const barDetail = needsApply
    ? `${active} · se aplicarán en el próximo inicio del juego`
    : `${active} · perfil aplicado a ${game.name}`;

  return (
    <>
      <PageLayout
        eyebrow="Biblioteca privada"
        title={
          <ProfilePicker
            name={draft.name}
            profiles={lib?.profiles ?? []}
            applied={applied}
            disabled={!lib || action.busy}
            onOpen={(name) => draft.open(name)}
            onCreate={() => setDialog("profile")}
          />
        }
        aside={
          <>
            <TextField
              className={styles.search}
              label="Buscar mod"
              icon={Search}
              type="search"
              placeholder="buscar mod…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <Button
              variant="outline"
              icon={FolderPlus}
              onClick={() => setDialog("import")}
            >
              Importar
            </Button>
          </>
        }
        footer={
          <PendingChangesBar
            title={barTitle}
            detail={lib ? barDetail : "Leyendo biblioteca…"}
            highlighted={needsApply}
            canRevert={draft.diff.count > 0}
            canApply={needsApply && Boolean(runtime.data)}
            applying={action.pending === "apply"}
            onRevert={draft.revert}
            onApply={() => void apply()}
          />
        }
      >
        {library.error && (
          <InlineError
            action={
              <Button onClick={() => void library.reload().catch(() => {})}>
                Reintentar
              </Button>
            }
          >
            {library.error}
          </InlineError>
        )}
        {action.error && <InlineError>{action.error}</InlineError>}
        {lib && lib.issues.length > 0 && (
          <InlineError>
            {plural(lib.issues.length, "mod no supera", "mods no superan")} la
            verificación y no se pueden seleccionar:{" "}
            {lib.issues.map((issue) => issue.detail).join(" · ")}
          </InlineError>
        )}
        <ModTable
          mods={filtered}
          selected={draft.selected}
          changeOf={draft.diff.changeOf}
          disabled={action.busy || !draft.name}
          emptyText={
            library.loading && !lib
              ? "Leyendo biblioteca…"
              : query
                ? `No hay mods que coincidan con «${query}».`
                : "Esta biblioteca todavía no tiene mods. Importa una carpeta o un ZIP para empezar."
          }
          onToggle={draft.toggle}
        />
        {lib && lib.profiles.length === 0 && mods.length > 0 && !draft.name && (
          <p className={styles.note}>
            Crea un perfil desde el selector para elegir qué mods se cargan.
          </p>
        )}
      </PageLayout>

      {dialog === "import" && (
        <ImportModDialog
          gameId={game.backendId}
          gameName={game.name}
          onClose={() => setDialog(null)}
          onImported={(mod) => {
            setDialog(null);
            void library.reload().catch(() => {});
            if (draft.name) draft.toggle(mod.id, true);
            toast(`«${mod.name}» importado`, "success");
          }}
        />
      )}
      {dialog === "profile" && (
        <NewProfileDialog
          existing={lib?.profiles.map((p) => p.name) ?? []}
          onClose={() => setDialog(null)}
          onCreate={(name) => {
            setDialog(null);
            draft.open(name, draft.selected);
          }}
        />
      )}
    </>
  );
}
