import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SwapIcon` instead.
 *
 * @example
 * ```svelte
 * <Swap color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Swap: Component<IconComponentProps, {}, "">;
type Swap = ReturnType<typeof Swap>;
export default Swap;
