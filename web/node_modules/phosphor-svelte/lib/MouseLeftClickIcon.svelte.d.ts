import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <MouseLeftClickIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MouseLeftClickIcon: Component<IconComponentProps, {}, "">;
type MouseLeftClickIcon = ReturnType<typeof MouseLeftClickIcon>;
export default MouseLeftClickIcon;
