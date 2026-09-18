import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SneakerIcon` instead.
 *
 * @example
 * ```svelte
 * <Sneaker color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sneaker: Component<IconComponentProps, {}, "">;
type Sneaker = ReturnType<typeof Sneaker>;
export default Sneaker;
