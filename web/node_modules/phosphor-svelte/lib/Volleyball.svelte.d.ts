import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VolleyballIcon` instead.
 *
 * @example
 * ```svelte
 * <Volleyball color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Volleyball: Component<IconComponentProps, {}, "">;
type Volleyball = ReturnType<typeof Volleyball>;
export default Volleyball;
