import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <HandGrabbingIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandGrabbingIcon: Component<IconComponentProps, {}, "">;
type HandGrabbingIcon = ReturnType<typeof HandGrabbingIcon>;
export default HandGrabbingIcon;
