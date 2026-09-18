import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ProhibitIcon` instead.
 *
 * @example
 * ```svelte
 * <Prohibit color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Prohibit: Component<IconComponentProps, {}, "">;
type Prohibit = ReturnType<typeof Prohibit>;
export default Prohibit;
