import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VideoCameraSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <VideoCameraSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const VideoCameraSlash: Component<IconComponentProps, {}, "">;
type VideoCameraSlash = ReturnType<typeof VideoCameraSlash>;
export default VideoCameraSlash;
