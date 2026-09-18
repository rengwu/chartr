import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PepperIcon` instead.
 *
 * @example
 * ```svelte
 * <Pepper color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pepper: Component<IconComponentProps, {}, "">;
type Pepper = ReturnType<typeof Pepper>;
export default Pepper;
