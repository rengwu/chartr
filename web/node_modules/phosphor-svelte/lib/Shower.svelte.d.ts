import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShowerIcon` instead.
 *
 * @example
 * ```svelte
 * <Shower color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Shower: Component<IconComponentProps, {}, "">;
type Shower = ReturnType<typeof Shower>;
export default Shower;
