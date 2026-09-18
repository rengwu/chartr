import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GogglesIcon` instead.
 *
 * @example
 * ```svelte
 * <Goggles color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Goggles: Component<IconComponentProps, {}, "">;
type Goggles = ReturnType<typeof Goggles>;
export default Goggles;
