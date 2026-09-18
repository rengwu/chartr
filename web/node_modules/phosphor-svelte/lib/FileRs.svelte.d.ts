import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileRsIcon` instead.
 *
 * @example
 * ```svelte
 * <FileRs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileRs: Component<IconComponentProps, {}, "">;
type FileRs = ReturnType<typeof FileRs>;
export default FileRs;
