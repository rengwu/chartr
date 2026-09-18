import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CameraIcon` instead.
 *
 * @example
 * ```svelte
 * <Camera color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Camera: Component<IconComponentProps, {}, "">;
type Camera = ReturnType<typeof Camera>;
export default Camera;
