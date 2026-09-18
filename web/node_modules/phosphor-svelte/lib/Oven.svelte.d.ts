import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OvenIcon` instead.
 *
 * @example
 * ```svelte
 * <Oven color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Oven: Component<IconComponentProps, {}, "">;
type Oven = ReturnType<typeof Oven>;
export default Oven;
