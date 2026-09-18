import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DivideIcon` instead.
 *
 * @example
 * ```svelte
 * <Divide color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Divide: Component<IconComponentProps, {}, "">;
type Divide = ReturnType<typeof Divide>;
export default Divide;
