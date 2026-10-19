import { ArrowLeft, Play, Search } from "lucide-react";
import { useState, type ReactNode } from "react";
import { call } from "../../../api/client";
import type { ContextLabMode, LoaderLabMode } from "../../../api/types";
import { useNavigation } from "../../../app/navigation";
import { Button } from "../../../components/Button";
import { InlineError } from "../../../components/InlineError";
import { Tag } from "../../../components/Tag";
import { useAction } from "../../../hooks/useAction";
import { useAsync } from "../../../hooks/useAsync";
import { PageLayout } from "../PageLayout";
import { JsonView } from "./JsonView";
import styles from "./LabsPage.module.css";
import { useZzzSources } from "./useZzzSources";

const contextModes: { value: ContextLabMode; label: string }[] = [
  { value: "same_prefix", label: "Same prefix" },
  { value: "different_prefixes", label: "Different prefixes" },
];
const loaderModes: LoaderLabMode[] = [
  "baseline",
  "positive",
  "hook",
  "missing_target",
  "missing_dll",
  "wrong_nonce",
];

const loadRuntimes = () =>
  call("inspect_runtime_bridge", { zzmiId: null, librariesId: null });

/** Results of the lab commands, shown raw as evidence. */
function useLab() {
  const action = useAction<string>();
  const [result, setResult] = useState<{
    label: string;
    value: unknown;
  } | null>(null);
  async function run(label: string, operation: () => Promise<unknown>) {
    const value = await action.run(label, operation);
    if (value !== undefined) setResult({ label, value });
    return value;
  }
  return { ...action, result, run };
}

export function LabsPage() {
  const { openConfig } = useNavigation();
  const runtimes = useAsync(loadRuntimes, "lab-runtimes");
  const [runtime, setRuntime] = useState("");
  const options = runtimes.data?.runtimes ?? [];
  const protonScript = runtime || options[0]?.proton_script || "";

  return (
    <PageLayout
      eyebrow="Diagnostics · experimental"
      title="Laboratories"
      aside={
        <Button
          variant="outline"
          icon={ArrowLeft}
          onClick={() => openConfig("advanced")}
        >
          Advanced
        </Button>
      }
    >
      <div className={styles.warning}>
        <Tag>Experimental</Tag>
        <p>
          These tests run helpers inside Proton prefixes and may have side
          effects. Use them only for investigation; they are not required to
          play.
        </p>
      </div>

      <label className={styles.runtime}>
        <span>Explicit Proton runtime</span>
        <select
          value={protonScript}
          onChange={(event) => setRuntime(event.target.value)}
          disabled={!options.length}
        >
          {!options.length && (
            <option value="">
              {runtimes.loading ? "Scanning…" : "None available"}
            </option>
          )}
          {options.map((option) => (
            <option key={option.proton_script} value={option.proton_script}>
              {option.display_name}
            </option>
          ))}
        </select>
      </label>
      {runtimes.error && <InlineError>{runtimes.error}</InlineError>}

      <BridgeLab protonScript={protonScript} />
      <ContextLab protonScript={protonScript} />
      <LoaderLab protonScript={protonScript} />
      <PrefixEntryLab />
    </PageLayout>
  );
}

function LabSection({
  title,
  description,
  lab,
  children,
}: {
  title: string;
  description: string;
  lab: ReturnType<typeof useLab>;
  children: ReactNode;
}) {
  return (
    <section className={styles.lab}>
      <header>
        <h2 className={styles.labTitle}>{title}</h2>
        <p className={styles.labDescription}>{description}</p>
      </header>
      <div className={styles.controls}>{children}</div>
      {lab.error && <InlineError>{lab.error}</InlineError>}
      {lab.result && (
        <JsonView label={lab.result.label} value={lab.result.value} />
      )}
    </section>
  );
}

function Consent({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className={styles.consent}>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      I accept the side effects
    </label>
  );
}

function BridgeLab({ protonScript }: { protonScript: string }) {
  const lab = useLab();
  const sources = useZzzSources();
  return (
    <LabSection
      title="Runtime Bridge"
      description="Helper handshake with the ZZMI runtime inside an LXMI-owned prefix."
      lab={lab}
    >
      <Button
        icon={Search}
        disabled={lab.busy}
        onClick={() =>
          void lab.run("Bridge inspection", () =>
            call("inspect_runtime_bridge", sources),
          )
        }
      >
        Inspect
      </Button>
      <Button
        variant="outline"
        icon={Play}
        disabled={
          lab.busy || !protonScript || !sources.zzmiId || !sources.librariesId
        }
        onClick={() =>
          void lab.run("Bridge test", () =>
            call("run_runtime_bridge_test", {
              zzmiId: sources.zzmiId!,
              librariesId: sources.librariesId!,
              protonScript,
            }),
          )
        }
      >
        Run test
      </Button>
    </LabSection>
  );
}

function ContextLab({ protonScript }: { protonScript: string }) {
  const lab = useLab();
  const [mode, setMode] = useState<ContextLabMode>("same_prefix");
  const [consent, setConsent] = useState(false);
  const args = { protonScript, mode, sideEffectsAcknowledged: consent };
  const disabled = lab.busy || !protonScript || !consent;
  return (
    <LabSection
      title="Proton/Wine context"
      description="Check whether two processes share a wineserver, direct injection, and reentry."
      lab={lab}
    >
      <select
        className={styles.select}
        value={mode}
        onChange={(event) => setMode(event.target.value as ContextLabMode)}
      >
        {contextModes.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <Consent checked={consent} onChange={setConsent} />
      <Button
        variant="outline"
        disabled={disabled}
        onClick={() =>
          void lab.run("Context test", () => call("run_context_lab_test", args))
        }
      >
        Context
      </Button>
      <Button
        variant="outline"
        disabled={disabled}
        onClick={() =>
          void lab.run("Direct injection", () =>
            call("run_direct_context_test", args),
          )
        }
      >
        Direct injection
      </Button>
      <Button
        variant="outline"
        disabled={disabled}
        onClick={() =>
          void lab.run("Reentry", () => call("run_context_reentry_test", args))
        }
      >
        Reentry
      </Button>
    </LabSection>
  );
}

function LoaderLab({ protonScript }: { protonScript: string }) {
  const lab = useLab();
  const [mode, setMode] = useState<LoaderLabMode>("baseline");
  const [consent, setConsent] = useState(false);
  return (
    <LabSection
      title="Loader Lab"
      description="Load a test DLL into a controlled target executable using the official loader."
      lab={lab}
    >
      <Button
        icon={Search}
        disabled={lab.busy}
        onClick={() =>
          void lab.run("Loader Lab status", () => call("inspect_loader_lab"))
        }
      >
        Inspect
      </Button>
      <Button
        disabled={lab.busy}
        onClick={() =>
          void lab.run("Preparation", () =>
            call("prepare_loader_lab").then(() => call("inspect_loader_lab")),
          )
        }
      >
        Prepare
      </Button>
      <select
        className={styles.select}
        value={mode}
        onChange={(event) => setMode(event.target.value as LoaderLabMode)}
      >
        {loaderModes.map((value) => (
          <option key={value} value={value}>
            {value.replaceAll("_", " ")}
          </option>
        ))}
      </select>
      <Consent checked={consent} onChange={setConsent} />
      <Button
        variant="outline"
        icon={Play}
        disabled={lab.busy || !protonScript || !consent}
        onClick={() =>
          void lab.run(`Experiment ${mode}`, () =>
            call("run_loader_lab_experiment", {
              protonScript,
              mode,
              sideEffectsAcknowledged: consent,
            }),
          )
        }
      >
        Run
      </Button>
    </LabSection>
  );
}

type EntryReview = {
  experiment_id: string;
  review_token: string;
  consent_text: string;
};

function PrefixEntryLab() {
  const lab = useLab();
  const [experimentId, setExperimentId] = useState<string | null>(null);
  const [review, setReview] = useState<EntryReview | null>(null);
  const [consent, setConsent] = useState(false);

  async function loadHistory() {
    const history = await lab.run("History", () =>
      call("inspect_zzz_harmless_prefix_entry_history"),
    );
    const current = (
      history as
        | { current?: { experiment: { experiment_id: string } } | null }
        | undefined
    )?.current;
    setExperimentId(current?.experiment.experiment_id ?? null);
  }

  return (
    <LabSection
      title="Real prefix entry"
      description="Run a harmless helper inside the real ZZZ prefix with a verified recovery backup."
      lab={lab}
    >
      <Button
        icon={Search}
        disabled={lab.busy}
        onClick={() =>
          void lab.run("Environment qualification", () =>
            call("inspect_linux_environment_qualification"),
          )
        }
      >
        Environment
      </Button>
      <Button
        disabled={lab.busy}
        onClick={() =>
          void lab.run("Qualify executor", () =>
            call("qualify_zzz_prefix_executor"),
          )
        }
      >
        Qualify executor
      </Button>
      <Button disabled={lab.busy} onClick={() => void loadHistory()}>
        History
      </Button>
      <Button
        disabled={lab.busy}
        onClick={() =>
          void lab
            .run("Experiment created", () =>
              call("create_zzz_harmless_prefix_entry_experiment"),
            )
            .then((value) => {
              const created = value as
                { experiment: { experiment_id: string } } | undefined;
              if (created) setExperimentId(created.experiment.experiment_id);
            })
        }
      >
        Create experiment
      </Button>
      <Button
        variant="outline"
        disabled={lab.busy || !experimentId}
        onClick={() =>
          void lab
            .run("Review", () =>
              call("prepare_zzz_harmless_prefix_entry", {
                experimentId: experimentId!,
              }),
            )
            .then((value) =>
              setReview((value as EntryReview | undefined) ?? null),
            )
        }
      >
        Review
      </Button>
      {review && (
        <div className={styles.review}>
          <p>{review.consent_text}</p>
          <Consent checked={consent} onChange={setConsent} />
          <Button
            variant="outline"
            tone="rust"
            icon={Play}
            disabled={lab.busy || !consent}
            onClick={() =>
              void lab
                .run("Entry result", () =>
                  call("run_zzz_harmless_prefix_entry", {
                    experimentId: review.experiment_id,
                    reviewToken: review.review_token,
                    consentAccepted: true,
                  }),
                )
                .then(() => {
                  setReview(null);
                  setConsent(false);
                })
            }
          >
            Run entry
          </Button>
        </div>
      )}
    </LabSection>
  );
}
