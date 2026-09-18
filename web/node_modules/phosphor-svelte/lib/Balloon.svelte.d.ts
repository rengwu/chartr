import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BalloonIcon` instead.
 *
 * @example
 * ```svelte
 * <Balloon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Balloon: Component<IconComponentProps, {}, "">;
type Balloon = ReturnType<typeof Balloon>;
export default Balloon;
