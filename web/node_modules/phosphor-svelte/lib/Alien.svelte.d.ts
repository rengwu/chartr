import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlienIcon` instead.
 *
 * @example
 * ```svelte
 * <Alien color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Alien: Component<IconComponentProps, {}, "">;
type Alien = ReturnType<typeof Alien>;
export default Alien;
