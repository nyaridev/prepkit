import { useState } from "react";
import { api } from "../lib/api";
import { ImageIcon } from "./icons";

/** WebView2 can't draw these, so they get an icon instead of a preview. */
const NO_PREVIEW = /\.(tiff?)$/i;

/** An image preview on a checkerboard, so transparent pixels are visible. */
export function Thumb({ path, large = false }: { path: string; large?: boolean }) {
  const [failed, setFailed] = useState(false);
  const showIcon = failed || NO_PREVIEW.test(path);
  return (
    <span className={large ? "thumb thumb-large" : "thumb"}>
      {showIcon ? (
        <ImageIcon />
      ) : (
        <img
          src={api.fileSrc(path)}
          alt=""
          loading="lazy"
          decoding="async"
          draggable={false}
          onError={() => setFailed(true)}
        />
      )}
    </span>
  );
}
