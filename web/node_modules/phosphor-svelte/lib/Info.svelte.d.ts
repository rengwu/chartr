import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `InfoIcon` instead.
 *
 * @example
 * ```svelte
 * <Info color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Info: Component<IconComponentProps, {}, "">;
type Info = ReturnType<typeof Info>;
export default Info;
