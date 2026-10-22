import { useEffect, useState } from "react";
import { call } from "../../../api/client";
import type { PreviewView } from "../../../api/types";

// Previews never change for a mod id, so each image is requested once.
const images = new Map<string, Promise<string>>();

export function forgetPreviewImages(gameId: string) {
  for (const key of images.keys())
    if (key.startsWith(`${gameId}/`)) images.delete(key);
}

function request(gameId: string, modId: string, view: PreviewView) {
  const key = `${gameId}/${modId}/${view}`;
  let image = images.get(key);
  if (!image) {
    image = call("get_mod_preview", { gameId, modId, view });
    images.set(key, image);
    image.catch(() => images.delete(key));
  }
  return image;
}

/** Data URL of a generated preview; null while loading or when not ready. */
export function usePreviewImage(
  gameId: string,
  modId: string,
  view: PreviewView,
  ready: boolean,
): string | null {
  const key = `${gameId}/${modId}/${view}`;
  const [loaded, setLoaded] = useState<{ key: string; url: string } | null>(
    null,
  );
  useEffect(() => {
    if (!ready) return;
    let cancelled = false;
    request(gameId, modId, view).then(
      (url) => {
        if (!cancelled) setLoaded({ key: `${gameId}/${modId}/${view}`, url });
      },
      () => {},
    );
    return () => {
      cancelled = true;
    };
  }, [gameId, modId, view, ready]);
  return ready && loaded?.key === key ? loaded.url : null;
}
