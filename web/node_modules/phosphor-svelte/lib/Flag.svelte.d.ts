import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlagIcon` instead.
 *
 * @example
 * ```svelte
 * <Flag color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Flag: Component<IconComponentProps, {}, "">;
type Flag = ReturnType<typeof Flag>;
export default Flag;
