import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PantsIcon` instead.
 *
 * @example
 * ```svelte
 * <Pants color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pants: Component<IconComponentProps, {}, "">;
type Pants = ReturnType<typeof Pants>;
export default Pants;
