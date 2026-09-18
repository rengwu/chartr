import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileXlsIcon` instead.
 *
 * @example
 * ```svelte
 * <FileXls color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileXls: Component<IconComponentProps, {}, "">;
type FileXls = ReturnType<typeof FileXls>;
export default FileXls;
