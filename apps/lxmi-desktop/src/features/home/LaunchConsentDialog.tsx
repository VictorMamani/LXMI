import { Play } from "lucide-react";
import { useState } from "react";
import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import styles from "./LaunchConsentDialog.module.css";

/** Shown before the first «Play with mods»: what happens and the user's risk. */
export function LaunchConsentDialog({
  experimental,
  onAccept,
  onClose,
}: {
  experimental: boolean;
  onAccept: () => void;
  onClose: () => void;
}) {
  const [accepted, setAccepted] = useState(false);
  return (
    <Dialog
      eyebrow="Before playing with mods"
      title={
        experimental ? "Experimental WWMI Direct Inject" : "How mods are loaded"
      }
      onClose={onClose}
      width={520}
      actions={
        <>
          <Button onClick={onClose}>Back</Button>
          <Button
            variant="primary"
            icon={Play}
            disabled={!accepted}
            onClick={onAccept}
          >
            Continue
          </Button>
        </>
      }
    >
      {experimental ? (
        <>
          <p>
            LXMI opens Wuthering Waves through Steam with -dx11, waits for the
            game process, and loads WWMI using the official Direct Inject
            method. The game installation, Steam, and the Proton prefix are not
            modified.
          </p>
          <p>
            This method has not been validated in gameplay. DLL injection in an
            online game with ACE anti-cheat may be detected and have account
            consequences. Mod loading and account safety are not guaranteed. Use
            test accounts only.
          </p>
        </>
      ) : (
        <>
          <p>
            LXMI opens the game through Steam and, once HoYoPlay is running,
            prepares the official XXMI hook. For a few seconds, HoYoPlay also
            receives the DLL and rejects it. The game installation and Steam are
            not modified.
          </p>
          <p>
            Mods are third-party content in an online game: using them may
            violate the game terms. The decision and risk are yours.
          </p>
        </>
      )}
      <label className={styles.consent}>
        <input
          type="checkbox"
          checked={accepted}
          onChange={(e) => setAccepted(e.target.checked)}
        />
        I understand and want to continue
      </label>
    </Dialog>
  );
}
