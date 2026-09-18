import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RainbowIcon` instead.
 *
 * @example
 * ```svelte
 * <Rainbow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Rainbow: Component<IconComponentProps, {}, "">;
type Rainbow = ReturnType<typeof Rainbow>;
export default Rainbow;
