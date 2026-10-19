import { useEffect, useState } from "react";
import type { Game } from "../../domain/games";
import styles from "./BackgroundVideo.module.css";

function usePrefersReducedMotion(): boolean {
  const query = "(prefers-reduced-motion: reduce)";
  const [reduced, setReduced] = useState(
    () => window.matchMedia(query).matches,
  );
  useEffect(() => {
    const media = window.matchMedia(query);
    const onChange = () => setReduced(media.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);
  return reduced;
}

/**
 * Looping art of the selected game behind the home screen. The poster is
 * shown while the video loads, if WebM is unsupported, or with reduced motion.
 */
export function BackgroundVideo({ game }: { game: Game }) {
  const reducedMotion = usePrefersReducedMotion();
  return (
    <div className={styles.stage} aria-hidden>
      <div key={game.id} className={styles.layer}>
        {reducedMotion ? (
          <img className={styles.media} src={game.media.poster} alt="" />
        ) : (
          <video
            className={styles.media}
            src={game.media.background}
            poster={game.media.poster}
            autoPlay
            muted
            loop
            playsInline
            disablePictureInPicture
            preload="auto"
          />
        )}
      </div>
      <div className={styles.veil} />
      <div className={styles.fade} />
    </div>
  );
}
