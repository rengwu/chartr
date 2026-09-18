import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BombIcon` instead.
 *
 * @example
 * ```svelte
 * <Bomb color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bomb: Component<IconComponentProps, {}, "">;
type Bomb = ReturnType<typeof Bomb>;
export default Bomb;
