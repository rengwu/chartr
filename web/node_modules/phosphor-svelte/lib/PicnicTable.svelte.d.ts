import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PicnicTableIcon` instead.
 *
 * @example
 * ```svelte
 * <PicnicTable color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PicnicTable: Component<IconComponentProps, {}, "">;
type PicnicTable = ReturnType<typeof PicnicTable>;
export default PicnicTable;
