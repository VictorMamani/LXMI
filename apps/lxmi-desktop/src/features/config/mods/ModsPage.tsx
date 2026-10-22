import { FolderPlus, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { errorMessage } from "../../../api/errors";
import { Button } from "../../../components/Button";
import { InlineError } from "../../../components/InlineError";
import { TextField } from "../../../components/TextField";
import { plural } from "../../../domain/format";
import {
  kindFilters,
  matchesKind,
  matchesQuery,
  sameModSelection,
  type KindFilter,
} from "../../../domain/mods";
import { useAction } from "../../../hooks/useAction";
import { readStored, writeStored } from "../../../lib/storage";
import { saveAndApplyProfile } from "../../../services/profileActivation";
import { useGameData } from "../../../state/GameDataContext";
import { useToast } from "../../../state/ToastContext";
import { PageLayout } from "../PageLayout";
import { ImportModDialog } from "./ImportModDialog";
import { ModGallery } from "./ModGallery";
import { ModTable } from "./ModTable";
import { ModToolbar, type ModLayout } from "./ModToolbar";
import styles from "./ModsPage.module.css";
import { NewProfileDialog } from "./NewProfileDialog";
import { PendingChangesBar } from "./PendingChangesBar";
import { ProfilePicker } from "./ProfilePicker";
import { useModInsights } from "./useModInsights";
import { useProfileDraft } from "./useProfileDraft";

const LAYOUT_KEY = "lxmi.mods.layout";

export function ModsPage() {
  const { game, library, runtime, refresh } = useGameData();
  const toast = useToast();
  const action = useAction<"apply">();
  const [query, setQuery] = useState("");
  const [dialog, setDialog] = useState<"import" | "profile" | null>(null);
  const [filter, setFilter] = useState<KindFilter>("all");
  const [layout, setLayoutState] = useState<ModLayout>(() =>
    readStored(LAYOUT_KEY) === "list" ? "list" : "gallery",
  );
  function setLayout(value: ModLayout) {
    writeStored(LAYOUT_KEY, value);
    setLayoutState(value);
  }

  const lib = library.data;
  const status = runtime.data?.status ?? null;
  const applied = status?.selection.current?.profile_name ?? null;
  const draft = useProfileDraft(lib, applied);

  const insights = useModInsights(game.backendId, lib?.mods ?? null);
  const details = insights.details;

  const mods = useMemo(() => lib?.mods ?? [], [lib]);
  const searched = mods.filter((mod) =>
    matchesQuery(mod, details.get(mod.id), query),
  );
  const filtered = searched.filter((mod) =>
    matchesKind(details.get(mod.id), filter),
  );
  const counts = Object.fromEntries(
    kindFilters.map(({ id }) => [
      id,
      searched.filter((mod) => matchesKind(details.get(mod.id), id)).length,
    ]),
  ) as Record<KindFilter, number>;
  const readyPreviews = mods.filter(
    (mod) => details.get(mod.id)?.preview === "ready",
  ).length;
  const filterLabel = kindFilters
    .find((f) => f.id === filter)
    ?.label.toLocaleLowerCase();

  async function clearPreviews() {
    try {
      await insights.clearPreviews();
      toast("Previews deleted", "success");
    } catch (reason) {
      toast(errorMessage(reason), "error");
    }
  }
  const changes = draft.diff.count + (draft.isSaved ? 0 : 1);
  const appliedMatches =
    status?.applied_profile?.name === draft.name &&
    sameModSelection(draft.selected, status.applied_profile.mod_ids);
  const needsApply = draft.name !== null && (changes > 0 || !appliedMatches);

  async function apply() {
    if (!lib || !runtime.data || !draft.name) return;
    const result = await action.run("apply", async () => {
      try {
        return await saveAndApplyProfile({
          gameId: game.backendId,
          library: lib,
          profileName: draft.name!,
          modIds: draft.selected,
          changed: changes > 0,
          status: runtime.data!.status,
          sources: runtime.data!.sources,
        });
      } finally {
        // Saving and activating are separate commits. Refresh even when the
        // second stage fails, so a retry uses the persisted library revision.
        await refresh().catch(() => {});
      }
    });
    if (!result) return;
    draft.commit();
    if (result.applied) toast(`Profile '${draft.name}' applied`, "success");
    else toast(`Profile saved. ${result.reason}`, "error");
  }

  const emptyText =
    library.loading && !lib
      ? "Reading library…"
      : query
        ? `No mods match '${query}'.`
        : filter !== "all"
          ? `No ${filterLabel} mods in this library.`
          : "This library has no mods yet. Import a directory or ZIP archive to get started.";

  let barTitle = "No pending changes";
  if (draft.diff.count > 0)
    barTitle = `${plural(draft.diff.count, "change")} pending`;
  else if (!draft.isSaved && draft.name) barTitle = "New profile not saved";
  else if (needsApply) barTitle = "Profile not applied";

  const active = `${draft.selected.length} of ${plural(mods.length, "mod")} selected`;
  const barDetail = needsApply
    ? `${active} · will load on the next game launch`
    : `${active} · profile applied to ${game.name}`;

  return (
    <>
      <PageLayout
        eyebrow="Private library"
        title={
          <ProfilePicker
            name={draft.name}
            profiles={lib?.profiles ?? []}
            applied={
              lib?.profiles.some(
                (profile) =>
                  profile.name === applied &&
                  status?.applied_profile &&
                  sameModSelection(
                    profile.mod_ids,
                    status.applied_profile.mod_ids,
                  ),
              )
                ? applied
                : null
            }
            disabled={!lib || action.busy}
            onOpen={(name) => draft.open(name)}
            onCreate={() => setDialog("profile")}
          />
        }
        aside={
          <>
            <TextField
              className={styles.search}
              label="Search mods"
              icon={Search}
              type="search"
              placeholder="search mods…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <Button
              variant="outline"
              icon={FolderPlus}
              disabled={action.busy}
              onClick={() => setDialog("import")}
            >
              Import
            </Button>
          </>
        }
        footer={
          <PendingChangesBar
            title={barTitle}
            detail={lib ? barDetail : "Reading library…"}
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
                Retry
              </Button>
            }
          >
            {library.error}
          </InlineError>
        )}
        {action.error && <InlineError>{action.error}</InlineError>}
        {lib && lib.issues.length > 0 && (
          <InlineError>
            {plural(lib.issues.length, "mod failed", "mods failed")}{" "}
            verification and cannot be selected:{" "}
            {lib.issues.map((issue) => issue.detail).join(" · ")}
          </InlineError>
        )}
        {mods.length > 0 && (
          <ModToolbar
            filter={filter}
            counts={counts}
            onFilter={setFilter}
            layout={layout}
            onLayout={setLayout}
            previewsEnabled={insights.enabled}
            progress={insights.progress}
            readyPreviews={readyPreviews}
            onPreviewsChange={insights.setEnabled}
            onClearPreviews={() => void clearPreviews()}
          />
        )}
        {layout === "gallery" ? (
          <ModGallery
            gameId={game.backendId}
            mods={filtered}
            details={details}
            rendering={insights.progress?.current ?? null}
            queued={insights.running}
            selected={draft.selected}
            changeOf={draft.diff.changeOf}
            disabled={action.busy || !draft.name}
            emptyText={emptyText}
            onToggle={draft.toggle}
          />
        ) : (
          <ModTable
            mods={filtered}
            details={details}
            selected={draft.selected}
            changeOf={draft.diff.changeOf}
            disabled={action.busy || !draft.name}
            emptyText={emptyText}
            onToggle={draft.toggle}
          />
        )}
        {lib && lib.profiles.length === 0 && mods.length > 0 && !draft.name && (
          <p className={styles.note}>
            Create a profile from the picker to choose which mods to load.
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
            toast(`'${mod.name}' imported`, "success");
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
