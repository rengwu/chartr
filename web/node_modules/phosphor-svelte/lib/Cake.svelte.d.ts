import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CakeIcon` instead.
 *
 * @example
 * ```svelte
 * <Cake color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cake: Component<IconComponentProps, {}, "">;
type Cake = ReturnType<typeof Cake>;
export default Cake;
