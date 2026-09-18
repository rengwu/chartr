import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PinwheelIcon` instead.
 *
 * @example
 * ```svelte
 * <Pinwheel color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pinwheel: Component<IconComponentProps, {}, "">;
type Pinwheel = ReturnType<typeof Pinwheel>;
export default Pinwheel;
