import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WheelchairMotionIcon` instead.
 *
 * @example
 * ```svelte
 * <WheelchairMotion color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WheelchairMotion: Component<IconComponentProps, {}, "">;
type WheelchairMotion = ReturnType<typeof WheelchairMotion>;
export default WheelchairMotion;
