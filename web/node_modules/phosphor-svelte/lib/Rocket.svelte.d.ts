import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RocketIcon` instead.
 *
 * @example
 * ```svelte
 * <Rocket color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Rocket: Component<IconComponentProps, {}, "">;
type Rocket = ReturnType<typeof Rocket>;
export default Rocket;
