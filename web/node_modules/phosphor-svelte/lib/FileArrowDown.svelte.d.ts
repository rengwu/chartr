import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileArrowDownIcon` instead.
 *
 * @example
 * ```svelte
 * <FileArrowDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileArrowDown: Component<IconComponentProps, {}, "">;
type FileArrowDown = ReturnType<typeof FileArrowDown>;
export default FileArrowDown;
