import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClubIcon` instead.
 *
 * @example
 * ```svelte
 * <Club color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Club: Component<IconComponentProps, {}, "">;
type Club = ReturnType<typeof Club>;
export default Club;
