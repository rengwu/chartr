import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SkipForwardIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SkipForwardIcon: Component<IconComponentProps, {}, "">;
type SkipForwardIcon = ReturnType<typeof SkipForwardIcon>;
export default SkipForwardIcon;
