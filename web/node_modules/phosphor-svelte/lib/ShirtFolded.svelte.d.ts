import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShirtFoldedIcon` instead.
 *
 * @example
 * ```svelte
 * <ShirtFolded color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShirtFolded: Component<IconComponentProps, {}, "">;
type ShirtFolded = ReturnType<typeof ShirtFolded>;
export default ShirtFolded;
