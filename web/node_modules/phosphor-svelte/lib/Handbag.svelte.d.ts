import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandbagIcon` instead.
 *
 * @example
 * ```svelte
 * <Handbag color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Handbag: Component<IconComponentProps, {}, "">;
type Handbag = ReturnType<typeof Handbag>;
export default Handbag;
