import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileArchiveIcon` instead.
 *
 * @example
 * ```svelte
 * <FileArchive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileArchive: Component<IconComponentProps, {}, "">;
type FileArchive = ReturnType<typeof FileArchive>;
export default FileArchive;
