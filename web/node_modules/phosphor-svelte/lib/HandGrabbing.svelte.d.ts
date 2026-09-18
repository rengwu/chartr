import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandGrabbingIcon` instead.
 *
 * @example
 * ```svelte
 * <HandGrabbing color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandGrabbing: Component<IconComponentProps, {}, "">;
type HandGrabbing = ReturnType<typeof HandGrabbing>;
export default HandGrabbing;
