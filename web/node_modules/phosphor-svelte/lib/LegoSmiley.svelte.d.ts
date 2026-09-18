import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LegoSmileyIcon` instead.
 *
 * @example
 * ```svelte
 * <LegoSmiley color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LegoSmiley: Component<IconComponentProps, {}, "">;
type LegoSmiley = ReturnType<typeof LegoSmiley>;
export default LegoSmiley;
