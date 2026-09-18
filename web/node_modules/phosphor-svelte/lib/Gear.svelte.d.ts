import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GearIcon` instead.
 *
 * @example
 * ```svelte
 * <Gear color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Gear: Component<IconComponentProps, {}, "">;
type Gear = ReturnType<typeof Gear>;
export default Gear;
