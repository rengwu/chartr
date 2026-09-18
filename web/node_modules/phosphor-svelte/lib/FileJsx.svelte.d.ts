import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileJsxIcon` instead.
 *
 * @example
 * ```svelte
 * <FileJsx color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileJsx: Component<IconComponentProps, {}, "">;
type FileJsx = ReturnType<typeof FileJsx>;
export default FileJsx;
