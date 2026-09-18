import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LaptopIcon` instead.
 *
 * @example
 * ```svelte
 * <Laptop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Laptop: Component<IconComponentProps, {}, "">;
type Laptop = ReturnType<typeof Laptop>;
export default Laptop;
