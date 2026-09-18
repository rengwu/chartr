import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DropHalfBottomIcon` instead.
 *
 * @example
 * ```svelte
 * <DropHalfBottom color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DropHalfBottom: Component<IconComponentProps, {}, "">;
type DropHalfBottom = ReturnType<typeof DropHalfBottom>;
export default DropHalfBottom;
