import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlowerIcon` instead.
 *
 * @example
 * ```svelte
 * <Flower color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Flower: Component<IconComponentProps, {}, "">;
type Flower = ReturnType<typeof Flower>;
export default Flower;
