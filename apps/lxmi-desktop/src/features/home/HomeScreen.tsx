import { Settings } from "lucide-react";
import { useState } from "react";
import { call } from "../../api/client";
import { errorMessage } from "../../api/errors";
import { useNavigation } from "../../app/navigation";
import { MenuButton } from "../../components/MenuButton";
import { plural, timeAgo } from "../../domain/format";
import { isLaunchActive } from "../../domain/launch";
import { useHotkeys } from "../../hooks/useHotkeys";
import { useGameData } from "../../state/GameDataContext";
import { readLastLaunch, useModLaunch } from "../../state/LaunchContext";
import { useToast } from "../../state/ToastContext";
import { useRuntimeSummary } from "../../state/useRuntimeSummary";
import { BackgroundVideo } from "./BackgroundVideo";
import { GameCards } from "./GameCards";
import { LaunchConsentDialog } from "./LaunchConsentDialog";
import { LaunchProgress } from "./LaunchProgress";
import { PlayActions } from "./PlayActions";
import { ShortcutsDialog } from "./ShortcutsDialog";
import styles from "./HomeScreen.module.css";

export function HomeScreen() {
  const { game, openConfig } = useNavigation();
  const { runtime } = useGameData();
  const summary = useRuntimeSummary();
  const launch = useModLaunch(game.backendId);
  const toast = useToast();
  const [dialog, setDialog] = useState<"consent" | "shortcuts" | null>(null);

  useHotkeys({ m: () => openConfig("mods") });

  const status = summary.status;
  const lastLaunch = readLastLaunch(game.backendId);
  const meta = [
    summary.profileName && `Profile '${summary.profileName}'`,
    summary.profileName && plural(summary.modCount, "mod"),
    summary.importerVersion && `${game.importer} ${summary.importerVersion}`,
    summary.librariesVersion && `XXMI Libraries ${summary.librariesVersion}`,
    summary.steam?.proton,
    lastLaunch && `last launch ${timeAgo(lastLaunch)}`,
  ].filter(Boolean);

  const active = isLaunchActive(launch.status) || launch.starting;

  let hint: string | null = null;
  if (launch.error) hint = launch.error;
  else if (launch.busyElsewhere) hint = "Another game is launching with mods.";
  else if (runtime.error) hint = runtime.error;
  else if (status && !status.launch_with_mods_available)
    hint = status.launch_blockers[0] ?? "Apply a profile to play with mods.";

  function play() {
    launch.clearError();
    if (launch.consentGiven()) void launch.start();
    else setDialog("consent");
  }

  async function playWithout() {
    try {
      await call("launch_game_without_mods", { gameId: game.backendId });
      toast("Request sent to Steam · without mods", "success");
    } catch (reason) {
      toast(errorMessage(reason), "error");
    }
  }

  return (
    <div className={styles.screen}>
      <BackgroundVideo game={game} />

      <header className={styles.top}>
        <GameCards />
        <MenuButton
          variant="glass"
          items={[
            { label: "Preferences", onSelect: () => openConfig("spaces") },
            { label: "Mods directory", onSelect: () => openConfig("spaces") },
            { label: "Shortcuts", onSelect: () => setDialog("shortcuts") },
          ]}
          trigger={(props) => (
            <button
              type="button"
              className={styles.settings}
              aria-label="Settings"
              {...props}
            >
              <Settings size={22} strokeWidth={1.75} aria-hidden />
            </button>
          )}
        />
      </header>

      <main className={styles.bottom}>
        <section className={styles.identity} aria-live="polite">
          <p className={styles.channel}>
            <span className={styles.channelMark} aria-hidden />
            {game.channel}&nbsp;&nbsp;{game.name.toUpperCase()}
          </p>
          <h1 key={game.id} className={styles.title}>
            {game.name}
          </h1>
          <p className={styles.meta}>
            {summary.loading && !status
              ? "Checking the mod runtime…"
              : meta.join(" · ")}
          </p>
        </section>

        {launch.visible && launch.status ? (
          <LaunchProgress
            status={launch.status}
            lastReached={launch.lastReached}
            showQualityHint={game.id === "zzz"}
            onCancel={() => void launch.cancel()}
            onRetry={play}
            onDismiss={launch.dismiss}
          />
        ) : (
          <PlayActions
            canPlayWithMods={
              Boolean(status?.launch_with_mods_available) &&
              !launch.busyElsewhere
            }
            canPlayWithout={Boolean(status?.installed)}
            playing={active}
            hint={hint}
            experimental={Boolean(status?.launch_experimental)}
            onPlay={play}
            onPlayWithout={() => void playWithout()}
            onManage={() => openConfig("mods")}
          />
        )}
      </main>

      {dialog === "consent" && (
        <LaunchConsentDialog
          key={game.id}
          experimental={Boolean(status?.launch_experimental)}
          onClose={() => setDialog(null)}
          onAccept={() => {
            launch.giveConsent();
            setDialog(null);
            void launch.start();
          }}
        />
      )}
      {dialog === "shortcuts" && (
        <ShortcutsDialog onClose={() => setDialog(null)} />
      )}
    </div>
  );
}
