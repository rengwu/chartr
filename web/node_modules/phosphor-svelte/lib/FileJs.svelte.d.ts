import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileJsIcon` instead.
 *
 * @example
 * ```svelte
 * <FileJs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileJs: Component<IconComponentProps, {}, "">;
type FileJs = ReturnType<typeof FileJs>;
export default FileJs;
