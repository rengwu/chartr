import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FinnTheHumanIcon` instead.
 *
 * @example
 * ```svelte
 * <FinnTheHuman color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FinnTheHuman: Component<IconComponentProps, {}, "">;
type FinnTheHuman = ReturnType<typeof FinnTheHuman>;
export default FinnTheHuman;
