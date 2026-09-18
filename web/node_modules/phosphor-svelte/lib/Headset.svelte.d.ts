import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeadsetIcon` instead.
 *
 * @example
 * ```svelte
 * <Headset color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Headset: Component<IconComponentProps, {}, "">;
type Headset = ReturnType<typeof Headset>;
export default Headset;
