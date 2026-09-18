import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileAudioIcon` instead.
 *
 * @example
 * ```svelte
 * <FileAudio color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileAudio: Component<IconComponentProps, {}, "">;
type FileAudio = ReturnType<typeof FileAudio>;
export default FileAudio;
