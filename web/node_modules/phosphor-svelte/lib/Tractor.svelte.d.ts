import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TractorIcon` instead.
 *
 * @example
 * ```svelte
 * <Tractor color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tractor: Component<IconComponentProps, {}, "">;
type Tractor = ReturnType<typeof Tractor>;
export default Tractor;
