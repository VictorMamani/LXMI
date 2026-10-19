import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { ActionButton } from "./components/Chrome";
import {
  errorMessage,
  type LaunchStatus,
  type RuntimeStatus,
} from "./runtimeTypes";

const ACK_KEY = "lxmi.modLaunchAcknowledged.v1";
const steps: { phase: LaunchStatus["phase"][]; label: string }[] = [
  { phase: ["preparing"], label: "Copia del perfil" },
  {
    phase: ["opening_launcher", "waiting_for_launcher"],
    label: "Launcher desde Steam",
  },
  { phase: ["arming"], label: "Carga preparada" },
  { phase: ["waiting_for_play"], label: "Pulsa Jugar" },
  { phase: ["checking_stability"], label: "Mods cargados" },
  { phase: ["running"], label: "En juego" },
];

function readAck() {
  try {
    return window.localStorage.getItem(ACK_KEY) === "yes";
  } catch {
    return false;
  }
}
function writeAck() {
  try {
    window.localStorage.setItem(ACK_KEY, "yes");
  } catch {
    // Without storage the notice is shown again next time.
  }
}

export default function ModLaunchCard({
  gameBackendId,
  runtime,
  loading,
  launchingWithout,
  onLaunchWithout,
}: {
  gameBackendId: string;
  runtime: RuntimeStatus | null;
  loading: boolean;
  launchingWithout: boolean;
  onLaunchWithout: () => void;
}) {
  const [status, setStatus] = useState<LaunchStatus | null>(null);
  const [error, setError] = useState("");
  const [confirming, setConfirming] = useState(false);
  const [accepted, setAccepted] = useState(false);
  const [starting, setStarting] = useState(false);
  const timer = useRef<number | null>(null);

  const active =
    status !== null && status.game_id === gameBackendId && !status.finished;
  const shown =
    status !== null && status.game_id === gameBackendId ? status : null;

  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      try {
        const next = await invoke<LaunchStatus | null>("game_with_mods_status");
        if (cancelled) return;
        setStatus(next);
        if (next && !next.finished)
          timer.current = window.setTimeout(() => void poll(), 1000);
      } catch (reason) {
        if (!cancelled) setError(errorMessage(reason));
      }
    };
    void poll();
    return () => {
      cancelled = true;
      if (timer.current !== null) window.clearTimeout(timer.current);
    };
  }, [gameBackendId, starting]);

  async function start() {
    setError("");
    setConfirming(false);
    setStarting(true);
    try {
      setStatus(
        await invoke<LaunchStatus>("start_game_with_mods", {
          gameId: gameBackendId,
        }),
      );
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setStarting(false);
    }
  }
  function requestStart() {
    if (readAck()) void start();
    else setConfirming(true);
  }
  async function cancel() {
    try {
      await invoke<boolean>("cancel_game_with_mods");
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  const available = Boolean(runtime?.launch_with_mods_available);
  const experimental = Boolean(runtime?.launch_experimental);
  const currentStep = shown
    ? steps.findIndex((s) => s.phase.includes(shown.phase))
    : -1;
  const lastGood =
    shown?.phase === "failed" || shown?.phase === "cancelled"
      ? -1
      : currentStep;

  return (
    <div className="mod-launch">
      {!available &&
        !loading &&
        runtime &&
        runtime.launch_blockers.length > 0 && (
          <ul className="mod-launch-blockers">
            {runtime.launch_blockers.map((blocker) => (
              <li key={blocker}>{blocker}</li>
            ))}
          </ul>
        )}

      {confirming && (
        <div
          className="mod-launch-consent"
          role="dialog"
          aria-label="Antes de jugar con mods"
        >
          <p>
            {experimental
              ? "LXMI abrirá el juego desde Steam y, cuando el juego esté listo, inyectará WWMI directamente en su proceso (Direct Inject, el método por defecto de WWMI). No se modifica la instalación del juego ni Steam."
              : "LXMI abrirá el juego desde Steam y, cuando su launcher esté abierto, preparará la carga oficial de XXMI. Durante unos segundos el launcher también recibe la DLL y la rechaza. No se modifica la instalación del juego ni Steam."}
          </p>
          {experimental && (
            <p className="mod-launch-risk" role="note">
              Experimental: esta ruta no está validada en partida. Inyectar una
              DLL en un juego en línea con anticheat (Anti-Cheat Expert) puede
              ser detectado y tener consecuencias en tu cuenta. LXMI no puede
              garantizar que los mods carguen ni que la cuenta quede a salvo.
              Úsala solo con cuentas de prueba.
            </p>
          )}
          <p>
            Los mods son contenido de terceros en un juego online: su uso puede
            infringir las condiciones del juego. La decisión y el riesgo son
            tuyos.
          </p>
          <label>
            <input
              type="checkbox"
              checked={accepted}
              onChange={(e) => setAccepted(e.target.checked)}
            />
            Entiendo y quiero continuar
          </label>
          <div className="mod-launch-actions">
            <ActionButton
              variant="primary"
              disabled={!accepted}
              onClick={() => {
                writeAck();
                void start();
              }}
            >
              Continuar
            </ActionButton>
            <ActionButton variant="quiet" onClick={() => setConfirming(false)}>
              Volver
            </ActionButton>
          </div>
        </div>
      )}

      {shown && (
        <div
          className="mod-launch-status"
          data-phase={shown.phase}
          role="status"
          aria-live="polite"
        >
          <ol>
            {steps.map((step, index) => (
              <li
                key={step.label}
                data-state={
                  index < lastGood ||
                  (index === lastGood && shown.phase === "running")
                    ? "done"
                    : index === currentStep
                      ? "current"
                      : "pending"
                }
              >
                {step.label}
              </li>
            ))}
          </ol>
          <p className="mod-launch-message">{shown.message}</p>
          {shown.phase === "waiting_for_play" &&
            gameBackendId === "zenless-zone-zero" && (
              <p className="mod-launch-hint">
                En ZZZ, los mods de personajes necesitan Calidad de personajes:
                Alta y Animación de personajes de alta precisión: Desactivada.
              </p>
            )}
          {active && shown.phase !== "checking_stability" && (
            <ActionButton variant="small" onClick={() => void cancel()}>
              Cancelar
            </ActionButton>
          )}
        </div>
      )}
      {error && (
        <p className="mod-launch-error" role="alert">
          {error}
        </p>
      )}
      {experimental && !confirming && (
        <p className="mod-launch-risk" role="note">
          Experimental: Direct Inject de WWMI sin validar en partida. Solo con
          cuentas de prueba.
        </p>
      )}
      <div className="hero-play-wrap">
        <ActionButton
          variant="primary"
          icon="play"
          className="play-button"
          disabled={!available || active || starting || loading}
          onClick={requestStart}
        >
          {active ? "Iniciando con mods…" : "Jugar con mods"}
        </ActionButton>
        <ActionButton
          variant="quiet"
          disabled={
            !runtime?.installed || launchingWithout || active || loading
          }
          onClick={onLaunchWithout}
        >
          {launchingWithout ? "Enviando a Steam…" : "Jugar sin mods"}
        </ActionButton>
      </div>
    </div>
  );
}
