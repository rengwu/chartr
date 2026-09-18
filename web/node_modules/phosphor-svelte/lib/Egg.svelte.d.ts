import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EggIcon` instead.
 *
 * @example
 * ```svelte
 * <Egg color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Egg: Component<IconComponentProps, {}, "">;
type Egg = ReturnType<typeof Egg>;
export default Egg;
