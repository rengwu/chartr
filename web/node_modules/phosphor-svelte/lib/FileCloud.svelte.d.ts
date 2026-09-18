import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileCloudIcon` instead.
 *
 * @example
 * ```svelte
 * <FileCloud color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileCloud: Component<IconComponentProps, {}, "">;
type FileCloud = ReturnType<typeof FileCloud>;
export default FileCloud;
