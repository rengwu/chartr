import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToothIcon` instead.
 *
 * @example
 * ```svelte
 * <Tooth color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tooth: Component<IconComponentProps, {}, "">;
type Tooth = ReturnType<typeof Tooth>;
export default Tooth;
