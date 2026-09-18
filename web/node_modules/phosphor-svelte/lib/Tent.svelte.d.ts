import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TentIcon` instead.
 *
 * @example
 * ```svelte
 * <Tent color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tent: Component<IconComponentProps, {}, "">;
type Tent = ReturnType<typeof Tent>;
export default Tent;
