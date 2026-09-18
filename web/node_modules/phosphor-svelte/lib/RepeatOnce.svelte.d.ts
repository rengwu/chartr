import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RepeatOnceIcon` instead.
 *
 * @example
 * ```svelte
 * <RepeatOnce color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RepeatOnce: Component<IconComponentProps, {}, "">;
type RepeatOnce = ReturnType<typeof RepeatOnce>;
export default RepeatOnce;
