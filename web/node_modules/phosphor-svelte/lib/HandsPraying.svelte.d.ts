import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandsPrayingIcon` instead.
 *
 * @example
 * ```svelte
 * <HandsPraying color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandsPraying: Component<IconComponentProps, {}, "">;
type HandsPraying = ReturnType<typeof HandsPraying>;
export default HandsPraying;
