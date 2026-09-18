import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DressIcon` instead.
 *
 * @example
 * ```svelte
 * <Dress color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Dress: Component<IconComponentProps, {}, "">;
type Dress = ReturnType<typeof Dress>;
export default Dress;
