import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThreeDIcon` instead.
 *
 * @example
 * ```svelte
 * <ThreeD color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThreeD: Component<IconComponentProps, {}, "">;
type ThreeD = ReturnType<typeof ThreeD>;
export default ThreeD;
