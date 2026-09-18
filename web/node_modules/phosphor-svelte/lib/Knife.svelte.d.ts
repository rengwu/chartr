import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `KnifeIcon` instead.
 *
 * @example
 * ```svelte
 * <Knife color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Knife: Component<IconComponentProps, {}, "">;
type Knife = ReturnType<typeof Knife>;
export default Knife;
