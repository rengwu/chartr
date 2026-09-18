import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileVideoIcon` instead.
 *
 * @example
 * ```svelte
 * <FileVideo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileVideo: Component<IconComponentProps, {}, "">;
type FileVideo = ReturnType<typeof FileVideo>;
export default FileVideo;
