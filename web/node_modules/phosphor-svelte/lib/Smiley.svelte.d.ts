import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileyIcon` instead.
 *
 * @example
 * ```svelte
 * <Smiley color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Smiley: Component<IconComponentProps, {}, "">;
type Smiley = ReturnType<typeof Smiley>;
export default Smiley;
