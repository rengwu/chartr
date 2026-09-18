import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CoffeeIcon` instead.
 *
 * @example
 * ```svelte
 * <Coffee color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Coffee: Component<IconComponentProps, {}, "">;
type Coffee = ReturnType<typeof Coffee>;
export default Coffee;
