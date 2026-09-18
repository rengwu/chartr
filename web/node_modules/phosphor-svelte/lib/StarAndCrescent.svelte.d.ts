import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StarAndCrescentIcon` instead.
 *
 * @example
 * ```svelte
 * <StarAndCrescent color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StarAndCrescent: Component<IconComponentProps, {}, "">;
type StarAndCrescent = ReturnType<typeof StarAndCrescent>;
export default StarAndCrescent;
