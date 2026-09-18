import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SkipForwardCircleIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SkipForwardCircleIcon: Component<IconComponentProps, {}, "">;
type SkipForwardCircleIcon = ReturnType<typeof SkipForwardCircleIcon>;
export default SkipForwardCircleIcon;
