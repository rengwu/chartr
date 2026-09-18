import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToiletIcon` instead.
 *
 * @example
 * ```svelte
 * <Toilet color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Toilet: Component<IconComponentProps, {}, "">;
type Toilet = ReturnType<typeof Toilet>;
export default Toilet;
