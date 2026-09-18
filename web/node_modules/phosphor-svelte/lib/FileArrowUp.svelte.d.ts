import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileArrowUpIcon` instead.
 *
 * @example
 * ```svelte
 * <FileArrowUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileArrowUp: Component<IconComponentProps, {}, "">;
type FileArrowUp = ReturnType<typeof FileArrowUp>;
export default FileArrowUp;
