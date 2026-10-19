import { Play } from "lucide-react";
import { useState } from "react";
import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import styles from "./LaunchConsentDialog.module.css";

/** Shown before the first «Jugar con mods»: what happens and the user's risk. */
export function LaunchConsentDialog({
  onAccept,
  onClose,
}: {
  onAccept: () => void;
  onClose: () => void;
}) {
  const [accepted, setAccepted] = useState(false);
  return (
    <Dialog
      eyebrow="Antes de jugar con mods"
      title="Cómo se cargan los mods"
      onClose={onClose}
      width={520}
      actions={
        <>
          <Button onClick={onClose}>Volver</Button>
          <Button
            variant="primary"
            icon={Play}
            disabled={!accepted}
            onClick={onAccept}
          >
            Continuar
          </Button>
        </>
      }
    >
      <p>
        LXMI abrirá el juego desde Steam y, cuando HoYoPlay esté abierto,
        preparará la carga oficial de XXMI. Durante unos segundos HoYoPlay
        también recibe la DLL y la rechaza. No se modifica la instalación del
        juego ni Steam.
      </p>
      <p>
        Los mods son contenido de terceros en un juego online: su uso puede
        infringir las condiciones del juego. La decisión y el riesgo son tuyos.
      </p>
      <label className={styles.consent}>
        <input
          type="checkbox"
          checked={accepted}
          onChange={(e) => setAccepted(e.target.checked)}
        />
        Entiendo y quiero continuar
      </label>
    </Dialog>
  );
}
