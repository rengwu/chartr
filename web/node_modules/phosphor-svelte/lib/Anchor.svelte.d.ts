import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AnchorIcon` instead.
 *
 * @example
 * ```svelte
 * <Anchor color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Anchor: Component<IconComponentProps, {}, "">;
type Anchor = ReturnType<typeof Anchor>;
export default Anchor;
