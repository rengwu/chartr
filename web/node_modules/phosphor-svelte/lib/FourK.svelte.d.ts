import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FourKIcon` instead.
 *
 * @example
 * ```svelte
 * <FourK color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FourK: Component<IconComponentProps, {}, "">;
type FourK = ReturnType<typeof FourK>;
export default FourK;
