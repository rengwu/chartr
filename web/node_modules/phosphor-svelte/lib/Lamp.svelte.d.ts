import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LampIcon` instead.
 *
 * @example
 * ```svelte
 * <Lamp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lamp: Component<IconComponentProps, {}, "">;
type Lamp = ReturnType<typeof Lamp>;
export default Lamp;
