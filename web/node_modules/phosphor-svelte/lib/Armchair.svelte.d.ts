import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArmchairIcon` instead.
 *
 * @example
 * ```svelte
 * <Armchair color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Armchair: Component<IconComponentProps, {}, "">;
type Armchair = ReturnType<typeof Armchair>;
export default Armchair;
