import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <FileMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileMinus: Component<IconComponentProps, {}, "">;
type FileMinus = ReturnType<typeof FileMinus>;
export default FileMinus;
