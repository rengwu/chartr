import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `KeyholeIcon` instead.
 *
 * @example
 * ```svelte
 * <Keyhole color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Keyhole: Component<IconComponentProps, {}, "">;
type Keyhole = ReturnType<typeof Keyhole>;
export default Keyhole;
